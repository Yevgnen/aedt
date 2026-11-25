//! The keys that reach the providers, kept apart from the settings: one JSON
//! object of provider id to key in the app's data directory, readable and
//! writable by this user only. DeepSeek's own harness keeps its keys the same
//! way. A provider that keeps no key of its own takes the one its name's
//! environment variable holds.

use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use nsh_core::element::data_dir;

/// The keys kept, by provider id.
pub(crate) type Keys = BTreeMap<String, String>;

/// The file the keys are kept in, beside the app's `storage.json`.
const FILE: &str = "secrets.json";

/// The keys the app kept, while it kept a readable file.
pub(crate) fn load() -> Keys {
    let Some(path) = file() else {
        return Keys::new();
    };
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Keys::new(),
        Err(error) => {
            eprintln!("secrets: {} could not be read: {error}", path.display());
            return Keys::new();
        }
    };
    match serde_json::from_str(&text) {
        Ok(keys) => keys,
        Err(error) => {
            eprintln!("secrets: {} could not be read: {error}", path.display());
            Keys::new()
        }
    }
}

/// Keeps `keys`, or says why it could not.
pub(crate) fn save(keys: &Keys) {
    let Some(path) = file() else {
        return;
    };
    let text = match serde_json::to_string_pretty(keys) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("secrets: the keys could not be written: {error}");
            return;
        }
    };
    if let Some(dir) = path.parent() {
        if let Err(error) = fs::create_dir_all(dir) {
            eprintln!("secrets: {} could not be made: {error}", dir.display());
            return;
        }
        restrict(dir, 0o700);
    }
    if let Err(error) = write(&path, &text) {
        eprintln!("secrets: {} could not be written: {error}", path.display());
    }
}

/// `keys` in `path`, readable and writable by this user only: through a file
/// beside it, so a crash leaves the keys as they were.
fn write(path: &Path, text: &str) -> std::io::Result<()> {
    let temporary = path.with_extension("json.tmp");
    {
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let mut file = options.open(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
    }
    fs::rename(&temporary, path)?;
    restrict(path, 0o600);
    Ok(())
}

/// Gives `path` `mode`, on the platforms that have one.
#[cfg(unix)]
fn restrict(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
}

#[cfg(not(unix))]
fn restrict(_: &Path, _: u32) {}

/// The file the keys are kept in, beside the app's `storage.json`.
fn file() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join(FILE))
}
