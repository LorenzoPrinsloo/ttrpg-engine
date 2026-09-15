use crate::modules::bags::BagError;
use crate::modules::tables::TableError;
use std::fmt;
use std::path::PathBuf;

/* A table that could not be loaded. Every variant names a path, because the
   point of the message is to be able to open the file and fix it. */
#[derive(Debug)]
pub enum LoadError {
    NoDirectory { tried: Vec<PathBuf> },
    NoTables { root: PathBuf },
    Unreadable { path: PathBuf, error: std::io::Error },
    Syntax { path: PathBuf, error: toml::de::Error },
    Name { path: PathBuf },
    Bag { path: PathBuf, error: BagError },
    Table { path: PathBuf, error: TableError },
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
            Self::Unreadable { path, error } => write!(f, "{}: {error}", path.display()),
            /* toml's own message already carries the line, column and a caret. */
            Self::Syntax { path, error } => write!(f, "{}: {error}", path.display()),
            Self::Name { path } => write!(
                f,
                "{}: a table name cannot be empty or contain spaces, it gets typed at the prompt",
                path.display()
            ),
            Self::Bag { path, error } => write!(f, "{}: {error}", path.display()),
            Self::Table { path, error } => write!(f, "{}: {error}", path.display()),
        }
    }
}

impl std::error::Error for LoadError {}
