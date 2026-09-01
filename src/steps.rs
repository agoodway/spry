use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::SetupStep;
use crate::git;
use crate::host::{self, HostRunner};
use crate::sprite::{exec_argv, format_command, ExecOutput, SpriteClient};
use crate::template::{self, GitInfo, SetupVars};

pub fn resolve_git(git: Option<GitInfo>, branch: Option<&str>, cwd: &Path) -> GitInfo {
    let info = git.unwrap_or_else(|| git::git_info(cwd));
    template::apply_branch_override(info, branch)
}

pub fn write_template_verbose(out: &mut dyn Write, git: &GitInfo) -> Result<()> {
    match git.branch.as_deref() {
        Some(branch) => {
            writeln!(out, "branch: {branch}")?;
            writeln!(out, "branch_slug: {}", template::slug(branch))?;
        }
        None => {
            writeln!(out, "branch: absent")?;
            writeln!(out, "branch_slug: absent")?;
        }
    }
    match git.commit.as_deref() {
        Some(commit) => writeln!(out, "commit: {commit}")?,
        None => writeln!(out, "commit: absent")?,
    }
    match git.remote.as_deref() {
        Some(remote) => writeln!(out, "remote: {remote}")?,
        None => writeln!(out, "remote: absent")?,
    }
    Ok(())
}

fn host_env(name: &str, org: Option<&str>) -> Vec<(&'static str, String)> {
    let mut env = vec![("SPRITE", name.to_string())];
    if let Some(org) = org {
        env.push(("ORG", org.to_string()));
    }
    env
}

/// Run recipe steps (setup, start, or stop). `phase` is used in progress labels and errors.
#[allow(clippy::too_many_arguments)]
pub fn run_steps(
    steps: &[SetupStep],
    dry_run: bool,
    verbose: bool,
    phase: &str,
    client: &dyn SpriteClient,
    host: &dyn HostRunner,
    name: &str,
    org: Option<&str>,
    git: &GitInfo,
    out: &mut dyn Write,
) -> Result<()> {
    let total = steps.len();
    let vars = SetupVars {
        sprite: Some(name),
        org,
    };
    for (index, step) in steps.iter().enumerate() {
        let command = template::expand_in(step.command(), git, vars)
            .with_context(|| format!("{phase} command {} of {total} failed", index + 1))?;
        let line = if step.is_host() {
            host::format_host_command(&command)
        } else {
            format_command(&exec_argv(name, org, &command))
        };
        if dry_run {
            writeln!(out, "{line}")?;
            continue;
        }
        let i = index + 1;
        let label = if step.is_host() {
            format!("[host] {command}")
        } else {
            command.clone()
        };
        writeln!(out, "[{i}/{total}] {label}")?;
        if verbose {
            writeln!(out, "{line}")?;
        }
        out.flush()?;
        let result = if step.is_host() {
            host.run(&command, &host_env(name, org))
        } else {
            client.exec(name, org, &command)
        };
        match result {
            Ok(ExecOutput { stdout, stderr }) => {
                if !stdout.is_empty() {
                    write!(out, "{stdout}")?;
                }
                if !stderr.is_empty() {
                    write!(out, "{stderr}")?;
                }
            }
            Err(err) => {
                return Err(err).with_context(|| format!("{phase} command {i} of {total} failed"));
            }
        }
    }
    Ok(())
}
