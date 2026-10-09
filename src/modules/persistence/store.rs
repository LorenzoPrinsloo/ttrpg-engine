use std::ffi::OsString;
use std::fmt;
use std::io::{BufWriter, ErrorKind, Write};
use std::path::PathBuf;

use serde::Deserialize;
use tempfile::NamedTempFile;

use crate::modules::persistence::session::SessionFile;

// Where sessions live on disk, one `<name>.json` per game. The directory is
// handed in rather than looked up, so tests can point it anywhere.
pub(crate) struct SessionStore {
    dir: PathBuf,
}

impl SessionStore {
    pub(crate) fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    // The store at `MOTHERSHIP_SESSIONS`, or else in the platform's data
    // directory. Not beside the tables: those are checked in, and a save
    // after every draw would keep the working tree dirty.
    pub(crate) fn locate() -> Result<Self, StoreError> {
        sessions_dir(std::env::var_os("MOTHERSHIP_SESSIONS"), dirs::data_dir()).map(Self::new)
    }

    // The file a session name maps to. The name becomes part of a path, so
    // only plain names get through: nothing like `../elsewhere`.
    pub(crate) fn path(&self, name: &str) -> Result<PathBuf, StoreError> {
        let plain = !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

        match plain {
            true => Ok(self.dir.join(format!("{name}.json"))),
            false => Err(StoreError::Name {
                name: name.to_string(),
            }),
        }
    }

    // `None` when nothing has been saved under `name` yet: the first start of
    // a game, not an error.
    pub(crate) fn load(&self, name: &str) -> Result<Option<SessionFile>, StoreError> {
        let path = self.path(name)?;

        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(StoreError::Io { path, error }),
        };
        let json = |error| StoreError::Json {
            path: path.clone(),
            error,
        };

        let header: Header = serde_json::from_str(&source).map_err(json)?;
        if header.version > SessionFile::VERSION {
            return Err(StoreError::Version {
                path: path.clone(),
                found: header.version,
                supported: SessionFile::VERSION,
            });
        }

        serde_json::from_str(&source).map(Some).map_err(json)
    }

    // Written to a temporary file beside the real one, flushed to disk, then
    // renamed over it. The rename is atomic, so a save that fails part-way
    // leaves the previous one intact rather than a truncated file.
    pub(crate) fn save(&self, name: &str, session: &SessionFile) -> Result<PathBuf, StoreError> {
        let path = self.path(name)?;
        let io = |error| StoreError::Io {
            path: path.clone(),
            error,
        };

        std::fs::create_dir_all(&self.dir).map_err(io)?;

        // Same directory as the target: a rename across filesystems is a copy,
        // and no longer atomic.
        let mut file = NamedTempFile::new_in(&self.dir).map_err(io)?;

        let mut writer = BufWriter::new(file.as_file_mut());
        serde_json::to_writer_pretty(&mut writer, session).map_err(|error| StoreError::Json {
            path: path.clone(),
            error,
        })?;
        writer.flush().map_err(io)?;
        drop(writer);

        file.as_file().sync_all().map_err(io)?;
        file.persist(&path).map_err(|failed| io(failed.error))?;

        Ok(path)
    }
}

// Pure, so the order can be tested without touching the environment. An
// override is taken as given, like `MOTHERSHIP_TABLES`.
fn sessions_dir(from_env: Option<OsString>, data: Option<PathBuf>) -> Result<PathBuf, StoreError> {
    match (from_env, data) {
        (Some(from_env), _) => Ok(PathBuf::from(from_env)),
        (None, Some(data)) => Ok(data.join("ttrpg-engine").join("sessions")),
        (None, None) => Err(StoreError::NoDirectory),
    }
}

// Just the version of a session file, read before the rest. Unknown fields
// are ignored, so a file from a newer build would otherwise load with its new
// data quietly dropped, and the next save would erase it.
#[derive(Deserialize)]
struct Header {
    version: u32,
}

// A session that could not be saved or loaded. Like `LoadError`, every
// message that concerns a file starts with its path.
#[derive(Debug)]
pub(crate) enum StoreError {
    NoDirectory,
    Name {
        name: String,
    },
    Io {
        path: PathBuf,
        error: std::io::Error,
    },
    Json {
        path: PathBuf,
        error: serde_json::Error,
    },
    Version {
        path: PathBuf,
        found: u32,
        supported: u32,
    },
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDirectory => write!(
                f,
                "cannot find a data directory to keep sessions in, set MOTHERSHIP_SESSIONS to one"
            ),
            Self::Name { name } => write!(
                f,
                "{name:?} is not a session name, use letters, digits, `-` and `_`"
            ),
            Self::Io { path, error } => write!(f, "{}: {error}", path.display()),
            // serde_json's message already carries the line and column.
            Self::Json { path, error } => write!(f, "{}: {error}", path.display()),
            Self::Version {
                path,
                found,
                supported,
            } => write!(
                f,
                "{}: saved by a newer version (format {found}, this one reads up to {supported})",
                path.display()
            ),
        }
    }
}

impl std::error::Error for StoreError {}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use super::*;

    fn session(version: u32) -> SessionFile {
        SessionFile {
            version,
            config_hashes: BTreeMap::from([(
                "bags/artifacts.toml".to_string(),
                "abc123".to_string(),
            )]),
            bags: BTreeMap::new(),
            tables: BTreeMap::new(),
        }
    }

    fn read(path: &Path) -> SessionFile {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn a_session_is_saved_under_its_name() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionStore::new(dir.path());

        let path = store.save("game-1", &session(1)).unwrap();

        assert_eq!(path, dir.path().join("game-1.json"));
        assert_eq!(read(&path).config_hashes, session(1).config_hashes);
    }

    #[test]
    fn saving_again_replaces_the_previous_save() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionStore::new(dir.path());

        store.save("game", &session(1)).unwrap();
        let path = store.save("game", &session(2)).unwrap();

        assert_eq!(read(&path).version, 2);
    }

    #[test]
    fn a_missing_directory_is_created() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionStore::new(dir.path().join("sessions"));

        let path = store.save("game", &session(1)).unwrap();

        assert!(path.is_file());
    }

    #[test]
    fn no_temporary_file_is_left_behind() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionStore::new(dir.path());

        store.save("game", &session(1)).unwrap();
        store.save("game", &session(2)).unwrap();

        let files: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(files, ["game.json"]);
    }

    #[test]
    fn names_that_would_escape_the_directory_are_rejected() {
        let store = SessionStore::new("/sessions");

        for name in ["", "../game", "a/b", "game.json", "my game"] {
            let error = store.path(name).err().unwrap();
            assert!(
                error.to_string().contains("not a session name"),
                "{name:?}: {error}"
            );
        }
    }

    #[test]
    fn a_saved_session_loads_back() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionStore::new(dir.path());

        store.save("game", &session(1)).unwrap();
        let loaded = store.load("game").unwrap().unwrap();

        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.config_hashes, session(1).config_hashes);
    }

    #[test]
    fn a_session_never_saved_is_none() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionStore::new(dir.path().join("sessions"));

        assert!(store.load("game").unwrap().is_none());
    }

    #[test]
    fn a_broken_file_names_the_path_and_the_line() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        std::fs::write(
            dir.path().join("game.json"),
            "{\n  \"version\": 1,\n  \"bags\": {\n}",
        )
        .unwrap();

        let error = store.load("game").err().unwrap().to_string();

        assert!(error.contains("game.json"), "{error}");
        assert!(error.contains("line"), "{error}");
    }

    #[test]
    fn a_session_from_a_newer_version_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        store
            .save("game", &session(SessionFile::VERSION + 1))
            .unwrap();

        let error = store.load("game").err().unwrap().to_string();

        assert!(error.contains("newer version"), "{error}");
        assert!(
            error.contains(&format!("format {}", SessionFile::VERSION + 1)),
            "{error}"
        );
    }

    #[test]
    fn the_override_wins_over_the_data_directory() {
        let dir = sessions_dir(Some("/games".into()), Some("/data".into())).unwrap();

        assert_eq!(dir, Path::new("/games"));
    }

    #[test]
    fn sessions_default_to_their_own_folder_in_the_data_directory() {
        let dir = sessions_dir(None, Some("/data".into())).unwrap();

        assert_eq!(dir, Path::new("/data/ttrpg-engine/sessions"));
    }

    #[test]
    fn with_nowhere_to_go_the_error_names_the_override() {
        let error = sessions_dir(None, None).err().unwrap().to_string();

        assert!(error.contains("MOTHERSHIP_SESSIONS"), "{error}");
    }

    #[test]
    fn loading_checks_the_name_too() {
        let store = SessionStore::new("/sessions");

        let error = store.load("../game").err().unwrap().to_string();

        assert!(error.contains("not a session name"), "{error}");
    }
}
