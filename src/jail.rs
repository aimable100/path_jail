use std::path::{Component, Path, PathBuf};
use crate::error::JailError;


/// A filesystem sandbox that restricts paths to a root directory.
#[derive(Debug, Clone)]
pub struct Jail {
    root: PathBuf,
}

impl Jail {
    /// Create a jail rooted at the given directory.
    /// Canonicalizes the root immediately.
    /// Errors if root does not exist or is not a directory.
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, JailError> {
        let root = root.as_ref().canonicalize()?;
        if !root.is_dir() {
            return Err(JailError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Root must be a directory"
            )));
        }
        Ok(Self { root })
    }

    /// Returns the canonicalized root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Safely join a relative path to the jail root.
    ///
    /// Resolves `..` components, follows symlinks, verifies containment.
    /// Works even if the final path does not exist.
    #[must_use = "use the returned path, not the original input"]
    pub fn join<P: AsRef<Path>>(&self, relative: P) -> Result<PathBuf, JailError> {
        let path = relative.as_ref();
        if path.is_absolute() {
            return Err(JailError::InvalidPath("absolute paths not allowed".into()));
        }

        let mut current = self.root.clone();
        for component in path.components() {
            match component {
                Component::Normal(name) => {
                    current.push(name);
                    // If it exists, resolve symlinks and check bounds
                    if current.exists() {
                        current = self.verify_inside(current)?;
                    }
                }
                Component::ParentDir => {
                    current.pop();
                    // Check we haven't escaped the jail
                    if !current.starts_with(&self.root) {
                        return Err(JailError::EscapedRoot {
                            attempted: path.to_path_buf(),
                            root: self.root.clone(),
                        });
                    }
                    // Re-verify after pop (parent might be a symlink)
                    if current.exists() {
                        current = self.verify_inside(current)?;
                    }
                }
                Component::CurDir => {} // Ignore "."
                Component::RootDir | Component::Prefix(_) => {
                    return Err(JailError::InvalidPath("absolute components not allowed".into()));
                }
            }
        }

        Ok(current)
    }

    /// Verify a path is inside the jail.
    fn verify_inside(&self, path: PathBuf) -> Result<PathBuf, JailError> {
        let canonical = path.canonicalize()?;
        if !canonical.starts_with(&self.root) {
            return Err(JailError::EscapedRoot {
                attempted: path,
                root: self.root.clone(),
            });
        }
        Ok(canonical)
    }

    /// Verify an absolute path is inside the jail.
    /// Returns the canonicalized path if it's inside, otherwise an error.
    /// The path must exist.
    #[must_use = "use the returned path, not the original input"]
    pub fn contains<P: AsRef<Path>>(&self, absolute: P) -> Result<PathBuf, JailError> {
        let absolute = absolute.as_ref();
        if !absolute.is_absolute() {
            return Err(JailError::InvalidPath("path must be absolute".into()));
        }
        self.verify_inside(absolute.to_path_buf())
    }

}

impl AsRef<Path> for Jail {
    fn as_ref(&self) -> &Path {
        &self.root
    }
}