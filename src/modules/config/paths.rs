use std::path::{Path, PathBuf};

use crate::modules::config::error::LoadError;

// Where to look for the tables directory, in order. Pure, so the order can be
// tested without touching the filesystem.
pub(crate) fn candidates(cwd: &Path, exe: Option<&Path>) -> Vec<PathBuf> {
    const CONF_ROOT: &str = "config";

    // look in  working directory
    let mut tried: Vec<PathBuf> = vec![cwd.join(CONF_ROOT)];

    // look in binary's directory
    if let Some(dir) = exe.and_then(Path::parent) {
        tried.push(dir.join(CONF_ROOT));
    }

    // look in CARGO_MANIFEST_DIR directory
    tried.push(Path::new(env!("CARGO_MANIFEST_DIR")).join(CONF_ROOT));

    tried
}

pub(crate) fn resolve() -> Result<PathBuf, LoadError> {
    // An override is taken as given rather than added to the search: falling
    // back from it would quietly hide a typo.
    if let Some(dir) = std::env::var_os("MOTHERSHIP_TABLES") {
        let dir = PathBuf::from(dir);

        return match dir.is_dir() {
            true => Ok(dir),
            false => Err(LoadError::NoDirectory { tried: vec![dir] }),
        };
    }

    let cwd = std::env::current_dir().unwrap_or_default();
    let exe = std::env::current_exe().ok();
    let tried = candidates(&cwd, exe.as_deref());

    tried
        .iter()
        .find(|dir| looks_like_tables(dir))
        .cloned()
        .ok_or(LoadError::NoDirectory { tried })
}

// Holding `bags/` or `tables/` is what makes a directory ours. Plain
// `is_dir` would match `src/modules/tables`, the Table module's own source.
fn looks_like_tables(dir: &Path) -> bool {
    dir.join("bags").is_dir() || dir.join("tables").is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(paths: &[PathBuf]) -> Vec<String> {
        paths
            .iter()
            .map(|path| path.display().to_string())
            .collect()
    }

    #[test]
    fn the_working_directory_is_searched_before_the_binary() {
        let tried = candidates(Path::new("/work"), Some(Path::new("/bin/untitled")));

        assert_eq!(names(&tried)[..2], ["/work/config", "/bin/config"]);
    }

    #[test]
    fn a_source_directory_called_tables_is_not_a_tables_directory() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));

        assert!(looks_like_tables(&manifest.join("config")));
        assert!(!looks_like_tables(&manifest.join("src/modules/config")));
    }

    #[test]
    fn the_compiled_in_directory_is_always_the_last_resort() {
        let tried = candidates(Path::new("/work"), None);

        assert_eq!(tried.len(), 2);
        assert!(tried.last().unwrap().ends_with("config"));
    }
}
