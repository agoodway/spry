use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::sprite::ExecOutput;

/// Run a setup command on the host (not inside the sprite).
pub trait HostRunner {
    fn run(&self, command: &str, env: &[(&str, String)]) -> Result<ExecOutput>;
}

/// Production runner: `bash -lc` with inherited stdout/stderr.
#[derive(Debug, Default, Clone, Copy)]
pub struct CommandHost;

impl HostRunner for CommandHost {
    fn run(&self, command: &str, env: &[(&str, String)]) -> Result<ExecOutput> {
        let mut cmd = Command::new("bash");
        cmd.arg("-lc")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        for (key, value) in env {
            cmd.env(key, value);
        }
        let status = cmd
            .status()
            .context("failed to run host setup command (`bash -lc`)")?;
        if !status.success() {
            bail!(
                "host setup command failed with status {}",
                status.code().unwrap_or(-1)
            );
        }
        Ok(ExecOutput::default())
    }
}

/// Format a host command line for dry-run / verbose output.
pub fn format_host_command(command: &str) -> String {
    format!("host: {command}")
}

#[cfg(test)]
use std::cell::RefCell;

/// Recorded host setup call used by tests.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCall {
    pub command: String,
    pub env: Vec<(String, String)>,
}

/// Test double that records host commands and does not shell out.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct FakeHostRunner {
    calls: RefCell<Vec<HostCall>>,
    fail: Option<String>,
}

#[cfg(test)]
impl FakeHostRunner {
    pub fn fail(message: &str) -> Self {
        Self {
            fail: Some(message.to_string()),
            ..Self::default()
        }
    }

    pub fn calls(&self) -> Vec<HostCall> {
        self.calls.borrow().clone()
    }
}

#[cfg(test)]
impl HostRunner for FakeHostRunner {
    fn run(&self, command: &str, env: &[(&str, String)]) -> Result<ExecOutput> {
        self.calls.borrow_mut().push(HostCall {
            command: command.to_string(),
            env: env
                .iter()
                .map(|(k, v)| ((*k).to_string(), v.clone()))
                .collect(),
        });
        if let Some(message) = &self.fail {
            bail!("{message}");
        }
        Ok(ExecOutput::default())
    }
}
