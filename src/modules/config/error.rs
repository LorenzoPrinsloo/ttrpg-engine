use std::fmt;
use std::path::PathBuf;

/* A table that could not be loaded. Every message names a path, because the
   point of it is to be able to open the file and fix it. */
#[derive(Debug)]
pub enum LoadError {
    NoDirectory { tried: Vec<PathBuf> },
    NoTables { root: PathBuf },
    Name { path: PathBuf },
    Config(figment::Error),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDirectory { tried } => {
                writeln!(f, "no tables directory found, tried:")?;
                for path in tried {
                    writeln!(f, "  {}", path.display())?;
                }
                write!(f, "set MOTHERSHIP_TABLES to a directory holding bags/ and tables/")
            }
            Self::NoTables { root } => write!(
                f,
                "{}: no table files in bags/ or tables/",
                root.display()
            ),
            Self::Name { path } => write!(
                f,
                "{}: a table name cannot be empty or contain spaces, it gets typed at the prompt",
                path.display()
            ),
            /* figment's message already names the key and the file. */
            Self::Config(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for LoadError {}
