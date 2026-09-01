use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};

use crate::config::{self, SetupStep};
use crate::host::HostRunner;
use crate::sprite::{
    create_argv, format_command, list_argv, missing_cli_error, name_present, SpriteClient,
};
use crate::steps;
use crate::template::{self, GitInfo};
use crate::user_error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Start,
    Stop,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
        }
    }

    fn complete_label(self) -> &'static str {
        match self {
            Self::Start => "Start complete",
            Self::Stop => "Stop complete",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppOpts {
    pub sprite: Option<String>,
    pub org: Option<String>,
    pub dry_run: bool,
    pub config: Option<PathBuf>,
    pub verbose: bool,
    pub branch: Option<String>,
    pub(crate) search_root: Option<PathBuf>,
    pub(crate) git: Option<GitInfo>,
}

/// Load config, require the VM to exist, and run start or stop commands.
pub fn run(
    phase: Phase,
    opts: &AppOpts,
    cwd: &Path,
    client: &dyn SpriteClient,
    host: &dyn HostRunner,
    sprite_available: bool,
    out: &mut dyn Write,
) -> Result<()> {
    let started = Instant::now();
    let resolved = config::load(
        cwd,
        opts.config.as_deref(),
        opts.sprite.as_deref(),
        opts.org.as_deref(),
        opts.search_root.as_deref(),
    )?;
    let git = steps::resolve_git(opts.git.clone(), opts.branch.as_deref(), cwd);

    let name = resolved.name.clone().ok_or_else(|| {
        user_error(
            "No sprite name is set.",
            "set `name` in the recipe or pass `--sprite <name>`",
        )
    })?;
    let name = template::expand(&name, &git)?;
    template::reject_slash_in_name(&name)?;
    let org = resolved.org.as_deref();

    if opts.verbose {
        writeln!(out, "config: {}", resolved.path.display())?;
        writeln!(out, "name: {name}")?;
        match org {
            Some(org) => writeln!(out, "org: {org}")?,
            None => writeln!(out, "org: absent")?,
        }
        steps::write_template_verbose(out, &git)?;
    }

    if !sprite_available {
        return Err(missing_cli_error());
    }

    if opts.verbose {
        writeln!(out, "{}", format_command(&list_argv(org)))?;
    }

    let names = client.list(org).context("failed to list sprites")?;
    if !name_present(&names, &name) {
        let create_line = format_command(&create_argv(&name, org));
        return Err(user_error(
            format!("Sprite `{name}` does not exist."),
            format!("run `{create_line}` or `spry setup`"),
        ));
    }

    let steps_list: &[SetupStep] = match phase {
        Phase::Start => &resolved.start,
        Phase::Stop => &resolved.stop,
    };

    steps::run_steps(
        steps_list,
        opts.dry_run,
        opts.verbose,
        phase.label(),
        client,
        host,
        &name,
        org,
        &git,
        out,
    )?;

    let elapsed = started.elapsed().as_secs_f64();
    let n = steps_list.len();
    if opts.dry_run {
        writeln!(
            out,
            "Dry run complete for '{name}' — {n} command(s) in {elapsed:.2}s"
        )?;
    } else {
        writeln!(
            out,
            "{} for '{name}' — {n} command(s) in {elapsed:.2}s",
            phase.complete_label()
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::FakeHostRunner;
    use crate::sprite::{FakeSpriteClient, SpriteCall};
    use std::fs;
    use tempfile::TempDir;

    fn write_recipe(dir: &Path, body: &str) {
        fs::write(dir.join(".spry.yaml"), body).unwrap();
    }

    fn opts_for(dir: &Path) -> AppOpts {
        AppOpts {
            search_root: Some(dir.to_path_buf()),
            git: Some(GitInfo::default()),
            ..AppOpts::default()
        }
    }

    fn run_app(
        phase: Phase,
        dir: &Path,
        opts: AppOpts,
        client: &FakeSpriteClient,
        sprite_available: bool,
    ) -> Result<String> {
        let host = FakeHostRunner::default();
        run_app_host(phase, dir, opts, client, &host, sprite_available)
    }

    fn run_app_host(
        phase: Phase,
        dir: &Path,
        opts: AppOpts,
        client: &FakeSpriteClient,
        host: &FakeHostRunner,
        sprite_available: bool,
    ) -> Result<String> {
        let mut out = Vec::new();
        run(phase, &opts, dir, client, host, sprite_available, &mut out)?;
        Ok(String::from_utf8(out).unwrap())
    }

    fn both_lists_recipe() -> &'static str {
        "name: demo\nstart:\n  - echo up\n  - echo ready\nstop:\n  - echo down\n  - echo gone\n"
    }

    #[test]
    fn sprite_cli_missing() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), both_lists_recipe());
        let client = FakeSpriteClient::default();
        for phase in [Phase::Start, Phase::Stop] {
            let err = run_app(phase, dir.path(), opts_for(dir.path()), &client, false)
                .unwrap_err()
                .to_string();
            assert!(err.to_lowercase().contains("install"), "{err}");
        }
        assert!(client.calls().is_empty());
    }

    #[test]
    fn name_missing() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "start: []\nstop: []\n");
        let client = FakeSpriteClient::default();
        for phase in [Phase::Start, Phase::Stop] {
            let err = run_app(phase, dir.path(), opts_for(dir.path()), &client, true)
                .unwrap_err()
                .to_string();
            assert!(err.contains("name") && err.contains("--sprite"), "{err}");
        }
        assert!(client.calls().is_empty());
    }

    #[test]
    fn missing_vm_does_not_create() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), both_lists_recipe());
        let client = FakeSpriteClient::default();
        for phase in [Phase::Start, Phase::Stop] {
            let err = run_app(phase, dir.path(), opts_for(dir.path()), &client, true)
                .unwrap_err()
                .to_string();
            assert!(err.contains("does not exist"), "{err}");
            assert!(err.contains("sprite create"), "{err}");
            assert!(err.contains("spry setup"), "{err}");
        }
        assert!(!client.created());
        assert!(client.exec_commands().is_empty());
    }

    #[test]
    fn dry_run_still_requires_vm() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), both_lists_recipe());
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        for phase in [Phase::Start, Phase::Stop] {
            let err = run_app(phase, dir.path(), opts.clone(), &client, true)
                .unwrap_err()
                .to_string();
            assert!(err.contains("does not exist"), "{err}");
        }
        assert!(client.exec_commands().is_empty());
    }

    #[test]
    fn empty_list_on_existing_vm() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        for phase in [Phase::Start, Phase::Stop] {
            let printed = run_app(phase, dir.path(), opts_for(dir.path()), &client, true).unwrap();
            assert!(printed.contains("0 command"), "{printed}");
            assert!(printed.contains(phase.complete_label()), "{printed}");
        }
        assert!(client.exec_commands().is_empty());
        assert_eq!(client.calls().len(), 2);
        assert!(client
            .calls()
            .iter()
            .all(|c| matches!(c, SpriteCall::List { .. })));
    }

    #[test]
    fn runs_start_commands_not_stop() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), both_lists_recipe());
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let printed = run_app(
            Phase::Start,
            dir.path(),
            opts_for(dir.path()),
            &client,
            true,
        )
        .unwrap();
        assert!(printed.contains("[1/2]"), "{printed}");
        assert!(printed.contains("[2/2]"), "{printed}");
        assert!(printed.contains("Start complete"), "{printed}");
        assert_eq!(
            client.exec_commands(),
            vec!["echo up".to_string(), "echo ready".to_string()]
        );
    }

    #[test]
    fn runs_stop_commands_not_start() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), both_lists_recipe());
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let printed =
            run_app(Phase::Stop, dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(printed.contains("[1/2]"), "{printed}");
        assert!(printed.contains("[2/2]"), "{printed}");
        assert!(printed.contains("Stop complete"), "{printed}");
        assert_eq!(
            client.exec_commands(),
            vec!["echo down".to_string(), "echo gone".to_string()]
        );
    }

    #[test]
    fn dry_run_prints_without_exec() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nstart:\n  - sprite-env services start app\nstop:\n  - sprite-env services stop app\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let start = run_app(Phase::Start, dir.path(), opts.clone(), &client, true).unwrap();
        assert!(start.contains("sprite exec"), "{start}");
        assert!(start.contains("sprite-env services start app"), "{start}");
        assert!(!start.contains("sprite-env services stop app"), "{start}");
        assert!(!start.contains("sprite create"), "{start}");
        let stop = run_app(Phase::Stop, dir.path(), opts, &client, true).unwrap();
        assert!(stop.contains("sprite-env services stop app"), "{stop}");
        assert!(!stop.contains("sprite-env services start app"), "{stop}");
        assert!(client.exec_commands().is_empty());
    }

    #[test]
    fn host_start_step() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\norg: acme\nstart:\n  - host: echo starting {{sprite}}\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let host = FakeHostRunner::default();
        run_app_host(
            Phase::Start,
            dir.path(),
            opts_for(dir.path()),
            &client,
            &host,
            true,
        )
        .unwrap();
        assert!(client.exec_commands().is_empty());
        assert_eq!(host.calls()[0].command, "echo starting demo");
        assert!(host.calls()[0]
            .env
            .iter()
            .any(|(k, v)| k == "SPRITE" && v == "demo"));
    }

    #[test]
    fn host_stop_step() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\norg: acme\nstop:\n  - host: echo stopping {{sprite}}\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let host = FakeHostRunner::default();
        run_app_host(
            Phase::Stop,
            dir.path(),
            opts_for(dir.path()),
            &client,
            &host,
            true,
        )
        .unwrap();
        assert!(client.exec_commands().is_empty());
        assert_eq!(host.calls()[0].command, "echo stopping demo");
        assert!(host.calls()[0]
            .env
            .iter()
            .any(|(k, v)| k == "SPRITE" && v == "demo"));
    }

    #[test]
    fn templated_name() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: app-{{branch_slug}}\nstart:\n  - echo hi\nstop:\n  - echo bye\n",
        );
        let client = FakeSpriteClient::with_sprites(&["app-feat-x"]);
        let mut opts = opts_for(dir.path());
        opts.branch = Some("feat/x".to_string());
        run_app(Phase::Start, dir.path(), opts.clone(), &client, true).unwrap();
        run_app(Phase::Stop, dir.path(), opts, &client, true).unwrap();
        assert_eq!(
            client.exec_commands(),
            vec!["echo hi".to_string(), "echo bye".to_string()]
        );
    }

    #[test]
    fn fail_fast() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nstart:\n  - echo one\n  - echo two\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]).fail_exec_at(0, "nope");
        let err = run_app(
            Phase::Start,
            dir.path(),
            opts_for(dir.path()),
            &client,
            true,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("1 of 2"), "{err}");
        assert!(err.contains("start"), "{err}");
        assert_eq!(client.exec_commands(), vec!["echo one".to_string()]);
    }
}
