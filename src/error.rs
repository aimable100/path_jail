use std::fmt;
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug)]
pub enum JailError {
    EscapedRoot { attempted: PathBuf, root: PathBuf },
    InvalidPath(String),
    Io(std::io::Error),
}

impl fmt::Display for JailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EscapedRoot { attempted, root } => {
                write!(f, "path '{}' escapes jail root '{}'", 
                       attempted.display(), root.display())
            }
            Self::InvalidPath(reason) => write!(f, "invalid path: {}", reason),
            Self::Io(err) => write!(f, "io error: {}", err),
        }
    }
}

impl std::error::Error for JailError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for JailError {
    fn from(err: std::io::Error) -> Self {
        JailError::Io(err)
    }
}