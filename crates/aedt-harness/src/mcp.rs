//! Local MCP servers: a process is started for a server's tool list or a
//! call, its list is kept in memory for the harnesses after this one, and
//! it is ended once no list or call has used it for a while. The catalog on
//! disk is what a server listed the last time it ran, offered when it cannot
//! be reached now.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        Arc, Mutex as StdMutex, OnceLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use rig_harness::{
    prelude::*,
    rig_core::{
        message::ToolName,
        tool::{DynamicTool, ToolExecutionError},
    },
};
use rig_rmcp::{McpConnection, McpTool, client_config, rmcp};
use rmcp::ServiceExt;
use serde::{Deserialize, Serialize};

/// How long a server gets to start and answer its tool list.
const START_TIMEOUT: Duration = Duration::from_secs(60);

/// How long a server is kept after the last list or call that used it.
const IDLE_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// How long a list is offered before a harness asks the server again.
const LIST_TTL: Duration = Duration::from_secs(10 * 60);

/// How often the hub looks for servers that have been idle long enough.
const SWEEP_EVERY: Duration = Duration::from_secs(30);

/// What a server listed the last time it ran, with what started it: the tools
/// offered when it cannot be reached now.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Catalog {
    command: String,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    /// Its tools, as it listed them.
    tools: Vec<rmcp::model::Tool>,
}

impl Catalog {
    /// The catalog kept at `path`, while `server` still starts the way it says.
    fn read(path: &Path, server: &crate::McpServer) -> Option<Self> {
        let kept: Self = serde_json::from_str(&fs::read_to_string(path).ok()?).ok()?;
        let env: BTreeMap<String, String> = server.env.iter().cloned().collect();
        let same = kept.command == server.command && kept.args == server.args && kept.env == env;

        same.then_some(kept)
    }

    /// Keeps `tools` as the tools `server` lists, for the runs after this one.
    fn keep(path: &Path, server: &crate::McpServer, tools: &[rmcp::model::Tool]) {
        let catalog = Self {
            command: server.command.clone(),
            args: server.args.clone(),
            env: server.env.iter().cloned().collect(),
            tools: tools.to_vec(),
        };
        let Ok(text) = serde_json::to_string(&catalog) else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Err(error) = fs::write(path, text) {
            eprintln!("mcp: {}: failed to keep its catalog: {error}", server.name);
        }
    }
}

/// What the composer's status line reads of a server: its name, whether its
/// process runs, and whether the last attempt to start it failed.
struct ServerStatus {
    name: String,
    running: AtomicBool,
    failed: AtomicBool,
}

impl ServerStatus {
    fn new(name: String) -> Self {
        Self {
            name,
            running: AtomicBool::new(false),
            failed: AtomicBool::new(false),
        }
    }

    fn running(&self) {
        self.running.store(true, Ordering::Relaxed);
        self.failed.store(false, Ordering::Relaxed);
    }

    fn idle(&self) {
        self.running.store(false, Ordering::Relaxed);
        self.failed.store(false, Ordering::Relaxed);
    }

    fn failed(&self) {
        self.running.store(false, Ordering::Relaxed);
        self.failed.store(true, Ordering::Relaxed);
    }

    fn snapshot(&self) -> crate::McpStatus {
        let state = if self.running.load(Ordering::Relaxed) {
            crate::McpState::Running
        } else if self.failed.load(Ordering::Relaxed) {
            crate::McpState::Error
        } else {
            crate::McpState::Idle
        };

        crate::McpStatus {
            name: self.name.clone(),
            state,
        }
    }
}

/// One server, shared by every harness: what starts it, the tools its last
/// list produced, and the connection a list or a call uses while it is open.
struct Server {
    spec: crate::McpServer,
    /// Where its catalog is kept, which a list refreshes.
    catalog: PathBuf,
    /// Its tools as it listed them in this run, with when, or none before it
    /// has been listed.
    listed: StdMutex<Option<Listed>>,
    /// Its process and connection, made when a list or call needs one.
    open: tokio::sync::Mutex<Option<Arc<McpConnection>>>,
    /// One tool list at a time: two harnesses asking do not list twice.
    listing: tokio::sync::Mutex<()>,
    /// When a list or call last used it, for the idle sweep.
    used: StdMutex<Instant>,
    /// How many lists and calls are using it now: an idle sweep leaves it be.
    busy: AtomicUsize,
    /// What the composer's status line says of it.
    status: Arc<ServerStatus>,
}

impl Server {
    fn new(spec: crate::McpServer, catalog: PathBuf) -> Self {
        let status = Arc::new(ServerStatus::new(spec.name.clone()));
        Self {
            spec,
            catalog,
            listed: StdMutex::new(None),
            open: tokio::sync::Mutex::new(None),
            listing: tokio::sync::Mutex::new(()),
            used: StdMutex::new(Instant::now()),
            busy: AtomicUsize::new(0),
            status,
        }
    }

    fn cached(&self) -> Option<Vec<rmcp::model::Tool>> {
        self.listed.lock().ok().and_then(|listed| {
            listed
                .as_ref()
                .filter(|listed| listed.at.elapsed() < LIST_TTL)
                .map(|listed| listed.tools.clone())
        })
    }

    /// Keeps `tools` as what this server lists, in memory and on disk.
    fn keep(&self, tools: &[rmcp::model::Tool]) {
        if let Ok(mut listed) = self.listed.lock() {
            *listed = Some(Listed {
                tools: tools.to_vec(),
                at: Instant::now(),
            });
        }
        Catalog::keep(&self.catalog, &self.spec, tools);
    }

    fn touch(&self) {
        if let Ok(mut used) = self.used.lock() {
            *used = Instant::now();
        }
    }

    /// Marks this server in use until the returned guard is dropped, so an
    /// idle sweep cannot end a process a list or a call is using.
    fn busy(self: &Arc<Self>) -> Busy {
        self.busy.fetch_add(1, Ordering::Relaxed);
        Busy(Arc::clone(self))
    }

    /// The connection to the server, started now if none is open.
    async fn connected(&self) -> Result<Arc<McpConnection>, String> {
        let mut open = self.open.lock().await;
        if let Some(connection) = open.as_ref().filter(|open| !open.is_closed()) {
            let connection = Arc::clone(connection);
            drop(open);
            self.touch();
            self.status.running();

            return Ok(connection);
        }
        // A connection that ended on its own is dropped before another starts.
        open.take();

        let connection = match start(&self.spec).await {
            Ok(connection) => connection,
            Err(why) => {
                self.status.failed();

                return Err(why);
            }
        };
        let connection = Arc::new(connection);
        *open = Some(Arc::clone(&connection));
        drop(open);
        self.touch();
        self.status.running();

        Ok(connection)
    }

    /// The tools the server lists: what it listed in this run already, else a
    /// list now, which the next harness offers.
    async fn list(&self) -> Result<Vec<rmcp::model::Tool>, String> {
        let _listing = self.listing.lock().await;
        if let Some(tools) = self.cached() {
            return Ok(tools);
        }

        let connection = self.connected().await?;
        let tools = connection
            .list_tools(START_TIMEOUT)
            .await
            .map_err(|error| error.to_string())?;
        self.keep(&tools);

        Ok(tools)
    }

    /// The tools this server offers: what it listed in this run, or what it
    /// listed last time when it cannot be reached now.
    async fn tools(self: &Arc<Self>) -> Vec<rmcp::model::Tool> {
        let _busy = self.busy();
        match self.list().await {
            Ok(tools) => tools,
            Err(error) => {
                eprintln!("mcp: {}: failed to list its tools: {error}", self.spec.name);
                Catalog::read(&self.catalog, &self.spec)
                    .map(|kept| kept.tools)
                    .unwrap_or_default()
            }
        }
    }

    /// Ends the connection when no list or call has used it for `IDLE_TIMEOUT`,
    /// or when the server ended it on its own.
    async fn release_if_idle(&self) {
        if self.busy.load(Ordering::Relaxed) > 0 {
            return;
        }
        let idle = self
            .used
            .lock()
            .map(|used| used.elapsed() >= IDLE_TIMEOUT)
            .unwrap_or(false);

        let connection = {
            let mut open = self.open.lock().await;
            if self.busy.load(Ordering::Relaxed) > 0 {
                return;
            }
            let closed = open
                .as_ref()
                .is_some_and(|connection| connection.is_closed());
            if closed || idle { open.take() } else { None }
        };
        if let Some(connection) = connection {
            connection.close().await;
            self.status.idle();
        }
    }

    /// Ends the connection now: the server was edited or removed.
    async fn close(&self) {
        let connection = {
            let mut open = self.open.lock().await;
            open.take()
        };
        if let Some(connection) = connection {
            connection.close().await;
        }
        self.status.idle();
    }

    /// The tool `definition` is to the agent: named for its server, and calling
    /// the server's own name for it once the server runs.
    fn tool(self: &Arc<Self>, definition: &rmcp::model::Tool) -> Option<DynamicTool> {
        let name = ToolName::new(namespaced(&self.spec.name, &definition.name)).ok()?;
        let called = definition.clone();
        let offered = name.clone();
        let server = self.clone();

        Some(DynamicTool::new(
            name,
            called.description.as_deref().unwrap_or_default().to_owned(),
            serde_json::Value::Object(called.input_schema.as_ref().clone()),
            move |args| {
                let server = server.clone();
                let called = called.clone();
                let offered = offered.clone();

                Box::pin(async move {
                    let call = async move {
                        let _busy = server.busy();
                        let connection = server
                            .connected()
                            .await
                            .map_err(ToolExecutionError::other)?;
                        let tool = McpTool::new(called, &connection).with_name(offered);
                        let tool = DynamicTool::try_from(tool)
                            .map_err(|error| ToolExecutionError::other(error.to_string()))?;

                        tool.run(args).await
                    };

                    // The connection speaks tokio's I/O, whose context the
                    // agent's executor does not hold: each poll enters it.
                    rig_http::runtime::bind(call)
                        .map_err(|error| ToolExecutionError::other(error.to_string()))?
                        .await
                })
            },
        ))
    }
}

/// The tools a server listed, and when it did.
struct Listed {
    tools: Vec<rmcp::model::Tool>,
    at: Instant,
}

/// A guard that keeps a server from being swept while a list or a call uses it.
struct Busy(Arc<Server>);

impl Drop for Busy {
    fn drop(&mut self) {
        self.0.busy.fetch_sub(1, Ordering::Relaxed);
        self.0.touch();
    }
}

/// The servers every harness shares, by name: a process is started for a
/// list or a call, and ended once idle.
struct Hub {
    servers: StdMutex<BTreeMap<String, Arc<Server>>>,
}

impl Hub {
    fn new() -> Arc<Self> {
        let hub = Arc::new(Self {
            servers: StdMutex::new(BTreeMap::new()),
        });
        hub.sweep_in_background();

        hub
    }

    /// The servers `specs` names, in order: one already known is shared, and
    /// one whose command or catalog changed is started again.
    fn sync(&self, specs: &[crate::McpServer], dir: &Path) -> Vec<Arc<Server>> {
        let Ok(mut servers) = self.servers.lock() else {
            return Vec::new();
        };
        let mut out = Vec::with_capacity(specs.len());
        for spec in specs {
            let catalog = dir.join(format!("{}.json", spec.name));
            let kept = servers
                .get(&spec.name)
                .filter(|server| server.spec == *spec && server.catalog == catalog)
                .cloned();
            if let Some(server) = kept {
                out.push(server);
                continue;
            }

            let server = Arc::new(Server::new(spec.clone(), catalog));
            if let Some(old) = servers.insert(spec.name.clone(), Arc::clone(&server))
                && let Ok(runtime) = rig_http::runtime::handle()
            {
                runtime.spawn(async move { old.close().await });
            }
            out.push(server);
        }

        out
    }

    /// Ends every server no list or call has used for `IDLE_TIMEOUT`.
    async fn sweep(&self) {
        let servers: Vec<Arc<Server>> = match self.servers.lock() {
            Ok(servers) => servers.values().cloned().collect(),
            Err(_) => return,
        };
        for server in servers {
            server.release_if_idle().await;
        }
    }

    /// Sweeps for idle servers for as long as the app runs.
    fn sweep_in_background(self: &Arc<Self>) {
        let Ok(runtime) = rig_http::runtime::handle() else {
            eprintln!("mcp: no runtime to end idle servers");

            return;
        };
        let hub = Arc::downgrade(self);
        runtime.spawn(async move {
            loop {
                tokio::time::sleep(SWEEP_EVERY).await;
                let Some(hub) = hub.upgrade() else {
                    return;
                };
                hub.sweep().await;
            }
        });
    }
}

/// The servers every harness shares.
fn hub() -> Arc<Hub> {
    static HUB: OnceLock<Arc<Hub>> = OnceLock::new();

    HUB.get_or_init(Hub::new).clone()
}

/// A server and the tools this harness offers from it.
pub(crate) struct Prepared {
    server: Arc<Server>,
    tools: Vec<rmcp::model::Tool>,
}

impl Prepared {
    /// What this server is doing now, for the composer's status line.
    pub(crate) fn status(&self) -> crate::McpStatus {
        self.server.status.snapshot()
    }
}

/// The servers the app configured, with the tools each offers: what a harness
/// listed before, or what the server lists now, when a list first needs it.
pub(crate) fn prepare(servers: &[crate::McpServer], dir: &Path) -> Vec<Prepared> {
    let runtime = match rig_http::runtime::handle() {
        Ok(runtime) => runtime,
        Err(_) => {
            eprintln!("mcp: no runtime to ask the servers for their tools");

            return Vec::new();
        }
    };
    let servers = hub().sync(servers, dir);
    let mut tasks = Vec::with_capacity(servers.len());
    for server in &servers {
        let server = Arc::clone(server);
        tasks.push(runtime.spawn(async move { server.tools().await }));
    }
    let tools = runtime.block_on(async move {
        let mut out = Vec::with_capacity(tasks.len());
        for task in tasks {
            out.push(match task.await {
                Ok(tools) => tools,
                Err(error) => {
                    eprintln!("mcp: failed to list a server's tools: {error}");
                    Vec::new()
                }
            });
        }
        out
    });

    servers
        .into_iter()
        .zip(tools)
        .map(|(server, tools)| Prepared { server, tools })
        .collect()
}

/// Starts `server` and hands it the client's handshake.
async fn start(server: &crate::McpServer) -> Result<McpConnection, String> {
    let mut command = tokio::process::Command::new(&server.command);
    command
        .args(&server.args)
        .envs(server.env.iter().map(|(key, value)| (key, value)))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let transport = rmcp::transport::TokioChildProcess::new(command)
        .map_err(|error| format!("failed to start {:?}: {error}", server.command))?;
    let service = tokio::time::timeout(
        START_TIMEOUT,
        client_config("aedt", env!("CARGO_PKG_VERSION")).serve(transport),
    )
    .await
    .map_err(|_| {
        format!(
            "{:?} did not answer the handshake in {}s",
            server.command,
            START_TIMEOUT.as_secs()
        )
    })?
    .map_err(|error| format!("{:?} did not answer the handshake: {error}", server.command))?;

    Ok(McpConnection::new(service))
}

/// The name the agent knows the tool `tool` of `server` by: both, so two
/// servers' tools of one name stay apart, in the characters a model's tool
/// names are made of.
fn namespaced(server: &str, tool: &str) -> String {
    let mut name: String = format!("{server}__{tool}")
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
                character
            } else {
                '_'
            }
        })
        .collect();
    name.truncate(64);

    name
}

/// The tools of the servers the app configured, registered for the agent to call.
pub(crate) struct McpTools {
    servers: Arc<Vec<Prepared>>,
}

impl McpTools {
    pub(crate) fn new(servers: Arc<Vec<Prepared>>) -> Self {
        Self { servers }
    }
}

impl Plugin for McpTools {
    fn build(&self, app: &mut App) {
        for prepared in self.servers.iter() {
            for definition in &prepared.tools {
                let Some(tool) = prepared.server.tool(definition) else {
                    continue;
                };
                let (definition, handler, _) = tool.into_parts();
                app.world_mut()
                    .spawn_tool(definition, handler, ToolOptions::default());
            }
        }
    }
}
