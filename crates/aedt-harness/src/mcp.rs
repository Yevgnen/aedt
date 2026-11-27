//! Local MCP servers: the tools each listed the last time it ran are offered
//! as they were, and the process behind them starts only when a call first
//! needs one. A server the app has never run is asked once while the harness
//! starts, for the tools to offer at all.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
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

/// How long a server gets to start and answer while something waits on it.
const START_TIMEOUT: Duration = Duration::from_secs(60);

/// How long a call waits for the tool list that refreshes the catalog behind
/// it: a server slow to list it is left with the catalog it had.
const LIST_TIMEOUT: Duration = Duration::from_secs(10);

/// What a server listed the last time it ran, with what started it: the tools
/// the agent is offered before any process exists.
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

/// One server: what starts it, the tools its last run listed, and the
/// connection its calls share once one needs it.
pub(crate) struct Server {
    spec: crate::McpServer,
    listed: Vec<rmcp::model::Tool>,
    /// Where its catalog is kept, which a connection it makes refreshes.
    catalog: PathBuf,
    /// Its process and connection, made when a call first needs one.
    open: tokio::sync::Mutex<Option<Arc<McpConnection>>>,
}

impl Server {
    /// The connection to the server, started now if none is open.
    async fn connected(&self) -> Result<Arc<McpConnection>, ToolExecutionError> {
        let mut open = self.open.lock().await;
        if let Some(connection) = open.as_ref().filter(|open| !open.is_closed()) {
            return Ok(connection.clone());
        }

        let connection = Arc::new(start(&self.spec).await.map_err(ToolExecutionError::other)?);
        *open = Some(connection.clone());
        // What it lists now is what the next harness offers, even where this
        // one still offers the catalog it had.
        if let Ok(tools) = connection.list_tools(LIST_TIMEOUT).await {
            Catalog::keep(&self.catalog, &self.spec, &tools);
        }

        Ok(connection)
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
                    let connection = server.connected().await?;
                    let tool = McpTool::new(called, &connection).with_name(offered);
                    let tool = DynamicTool::try_from(tool)
                        .map_err(|error| ToolExecutionError::other(error.to_string()))?;

                    tool.run(args).await
                })
            },
        ))
    }
}

/// The servers the app configured, with the tools each lists: one already known
/// is offered from its catalog without anything running, and one never run is
/// started once here — the only way to learn what to offer — and kept in `dir`
/// for the runs after.
pub(crate) fn prepare(servers: &[crate::McpServer], dir: &Path) -> Vec<Arc<Server>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    let Ok(runtime) = runtime else {
        eprintln!("mcp: no runtime to ask the servers for their tools");

        return Vec::new();
    };

    servers
        .iter()
        .map(|server| {
            let catalog = dir.join(format!("{}.json", server.name));
            let listed = match Catalog::read(&catalog, server) {
                Some(kept) => kept.tools,
                None => runtime
                    .block_on(first_run(server, &catalog))
                    .unwrap_or_default(),
            };

            Arc::new(Server {
                spec: server.clone(),
                listed,
                catalog,
                open: tokio::sync::Mutex::new(None),
            })
        })
        .collect()
}

/// Starts `server` for the first time, keeps what it lists, and lets it go: the
/// connection a call makes later is the one that serves it.
async fn first_run(server: &crate::McpServer, catalog: &Path) -> Option<Vec<rmcp::model::Tool>> {
    let connection = match start(server).await {
        Ok(connection) => connection,
        Err(why) => {
            eprintln!("mcp: {}: {why}", server.name);

            return None;
        }
    };
    match connection.list_tools(START_TIMEOUT).await {
        Ok(tools) => {
            Catalog::keep(catalog, server, &tools);

            Some(tools)
        }
        Err(error) => {
            eprintln!("mcp: {}: failed to list its tools: {error}", server.name);

            None
        }
    }
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
    let service = client_config("aedt", env!("CARGO_PKG_VERSION"))
        .serve(transport)
        .await
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

/// The tools of the servers the app started, registered for the agent to call.
pub(crate) struct McpTools {
    servers: Vec<Arc<Server>>,
}

impl McpTools {
    pub(crate) fn new(servers: Vec<Arc<Server>>) -> Self {
        Self { servers }
    }
}

impl Plugin for McpTools {
    fn build(&self, app: &mut App) {
        for server in &self.servers {
            for definition in &server.listed {
                let Some(tool) = server.tool(definition) else {
                    continue;
                };
                let (definition, handler, _) = tool.into_parts();
                app.world_mut()
                    .spawn_tool(definition, handler, ToolOptions::default());
            }
        }
    }
}
