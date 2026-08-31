pub mod cli;
pub mod config;
pub mod init;
pub mod setup;
pub mod sprite;

use anyhow::anyhow;

/// Build a user-facing error that includes an actionable hint.
pub fn user_error(problem: impl std::fmt::Display, fix: impl std::fmt::Display) -> anyhow::Error {
    anyhow!("{problem}\n\nTo fix this: {fix}")
}
