//! A secure filesystem sandbox.
//!
//! Restricts paths to a root directory, preventing traversal attacks.

mod error;
mod jail;

pub use error::JailError;
pub use jail::Jail;