pub mod app;
pub mod cli;
pub mod config;
pub mod git;
pub mod host;
pub mod init;
pub mod setup;
pub mod sprite;
pub mod steps;
pub mod template;

use anyhow::anyhow;

/// Build a user-facing error that includes an actionable hint.
pub fn user_error(problem: impl std::fmt::Display, fix: impl std::fmt::Display) -> anyhow::Error {
    anyhow!("{problem}\n\nTo fix this: {fix}")
}
