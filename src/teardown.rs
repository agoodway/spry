use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};

use crate::config;
use crate::host::HostRunner;
use crate::sprite::{
    destroy_argv, format_command, list_argv, missing_cli_error, name_present, SpriteClient,
};
use crate::steps;
use crate::template::{self, GitInfo};
use crate::user_error;

#[derive(Debug, Clone, Default)]
pub struct TeardownOpts {
    pub sprite: Option<String>,
    pub org: Option<String>,
    pub dry_run: bool,
    pub config: Option<PathBuf>,
    pub verbose: bool,
    pub branch: Option<String>,
    pub no_destroy: bool,
    /// When set, config discovery does not walk above this directory (test seam).
    pub(crate) search_root: Option<PathBuf>,
    /// When set, skip `git_info(cwd)` (test seam).
    pub(crate) git: Option<GitInfo>,
}

/// Load config, require the VM to exist, run teardown commands, then destroy unless requested.
pub fn run(
    opts: &TeardownOpts,
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
        return Err(user_error(
            format!("Sprite `{name}` does not exist."),
            "check `--sprite` / recipe `name` and `--org`; if the VM is already gone, clean leftover host resources (for example deploy keys) manually",
        ));
    }

    steps::run_steps(
        &resolved.teardown,
        opts.dry_run,
        opts.verbose,
        "teardown",
        client,
        host,
        &name,
        org,
        &git,
        out,
    )?;

    if !opts.no_destroy {
        let destroy_line = format_command(&destroy_argv(&name, org));
        if opts.dry_run {
            writeln!(out, "{destroy_line}")?;
        } else {
            if opts.verbose {
                writeln!(out, "{destroy_line}")?;
            }
            out.flush()?;
            client
                .destroy(&name, org)
                .with_context(|| format!("failed to destroy sprite `{name}`"))?;
        }
    }

    let elapsed = started.elapsed().as_secs_f64();
    let status = match (opts.dry_run, opts.no_destroy) {
        (true, false) => "would be destroyed",
        (true, true) => "would be kept",
        (false, false) => "destroyed",
        (false, true) => "kept",
    };
    let n = resolved.teardown.len();
    if opts.dry_run {
        writeln!(
            out,
            "Dry run complete for '{name}' ({status}) — {n} command(s) in {elapsed:.2}s"
        )?;
    } else {
        writeln!(
            out,
            "Teardown complete for '{name}' ({status}) — {n} command(s) in {elapsed:.2}s"
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::FakeHostRunner;
    use crate::sprite::{exec_argv, FakeSpriteClient, SpriteCall};
    use std::fs;
    use tempfile::TempDir;

    fn write_recipe(dir: &Path, body: &str) {
        fs::write(dir.join(".spry.yaml"), body).unwrap();
    }

    fn opts_for(dir: &Path) -> TeardownOpts {
        TeardownOpts {
            search_root: Some(dir.to_path_buf()),
            git: Some(GitInfo::default()),
            ..TeardownOpts::default()
        }
    }

    fn git_info(branch: &str, commit: &str, remote: &str) -> GitInfo {
        GitInfo {
            branch: Some(branch.to_string()),
            commit: Some(commit.to_string()),
            remote: Some(remote.to_string()),
        }
    }

    fn run_teardown(
        dir: &Path,
        opts: TeardownOpts,
        client: &FakeSpriteClient,
        sprite_available: bool,
    ) -> Result<String> {
        let host = FakeHostRunner::default();
        run_teardown_host(dir, opts, client, &host, sprite_available)
    }

    fn run_teardown_host(
        dir: &Path,
        opts: TeardownOpts,
        client: &FakeSpriteClient,
        host: &FakeHostRunner,
        sprite_available: bool,
    ) -> Result<String> {
        let mut out = Vec::new();
        run(&opts, dir, client, host, sprite_available, &mut out)?;
        Ok(String::from_utf8(out).unwrap())
    }

    fn destroy_calls(client: &FakeSpriteClient) -> usize {
        client
            .calls()
            .iter()
            .filter(|c| matches!(c, SpriteCall::Destroy { .. }))
            .count()
    }

    #[test]
    fn teardown_commands_then_destroy() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nteardown:\n  - echo one\n  - echo two\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        run_teardown(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert_eq!(
            client.exec_commands(),
            vec!["echo one".to_string(), "echo two".to_string()]
        );
        assert!(client.destroyed());
        assert!(!client.created());
        match client.calls().last() {
            Some(SpriteCall::Destroy { name, org }) => {
                assert_eq!(name, "demo");
                assert_eq!(org, &None);
                let argv = destroy_argv(name, org.as_deref());
                assert_eq!(argv, vec!["destroy", "-s", "demo", "--force"]);
                assert!(argv.contains(&"-s".to_string()));
                assert!(argv.contains(&"--force".to_string()));
                assert!(!argv.contains(&"-f".to_string()));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn teardown_does_not_run_stop() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nstart:\n  - echo start-me\nstop:\n  - echo stop-me\nteardown:\n  - echo teardown-only\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        run_teardown(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert_eq!(
            client.exec_commands(),
            vec!["echo teardown-only".to_string()]
        );
        assert!(client.destroyed());
        assert!(!client.created());
    }

    #[test]
    fn destroy_includes_org_when_set() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\norg: acme\nteardown: []\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        run_teardown(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        match client.calls().last() {
            Some(SpriteCall::Destroy { name, org }) => {
                assert_eq!(name, "demo");
                assert_eq!(org.as_deref(), Some("acme"));
                assert_eq!(
                    destroy_argv(name, org.as_deref()),
                    vec!["destroy", "-s", "demo", "--force", "-o", "acme"]
                );
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn org_omitted_from_destroy() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        run_teardown(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        for call in client.calls() {
            match call {
                SpriteCall::List { org }
                | SpriteCall::Create { org, .. }
                | SpriteCall::Exec { org, .. }
                | SpriteCall::Destroy { org, .. } => {
                    assert_eq!(org, None);
                }
            }
        }
        match client
            .calls()
            .into_iter()
            .find(|c| matches!(c, SpriteCall::Destroy { .. }))
        {
            Some(SpriteCall::Destroy { name, org }) => {
                assert_eq!(name, "demo");
                assert_eq!(org, None);
                let argv = destroy_argv(&name, org.as_deref());
                assert!(!argv.contains(&"-o".to_string()));
                assert_eq!(argv, vec!["destroy", "-s", "demo", "--force"]);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn sprite_cli_missing() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown:\n  - echo hi\n");
        let client = FakeSpriteClient::default();
        let host = FakeHostRunner::default();
        let err = run_teardown_host(dir.path(), opts_for(dir.path()), &client, &host, false)
            .unwrap_err()
            .to_string();
        assert!(err.to_lowercase().contains("install"), "{err}");
        assert!(err.contains("sprite"), "{err}");
        assert!(err.contains("To fix this"), "{err}");
        assert!(client.calls().is_empty());
        assert!(!client.destroyed());
        assert!(client.exec_commands().is_empty());
        assert!(host.calls().is_empty());
    }

    #[test]
    fn name_missing() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "teardown: []\n");
        let client = FakeSpriteClient::default();
        let err = run_teardown(dir.path(), opts_for(dir.path()), &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("name") && err.contains("--sprite"), "{err}");
        assert!(err.contains("To fix this"), "{err}");
        assert!(client.calls().is_empty());
    }

    #[test]
    fn missing_vm_does_not_destroy() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nteardown:\n  - echo hi\n  - host: echo host-clean\n",
        );
        let client = FakeSpriteClient::default();
        let host = FakeHostRunner::default();
        let err = run_teardown_host(dir.path(), opts_for(dir.path()), &client, &host, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not exist"), "{err}");
        assert!(err.contains("--sprite") || err.contains("name"), "{err}");
        assert!(!err.contains("sprite create"), "{err}");
        assert!(client.exec_commands().is_empty());
        assert!(host.calls().is_empty());
        assert!(!client.destroyed());
        assert!(!client.created());
        assert!(matches!(
            client.calls().as_slice(),
            [SpriteCall::List { .. }]
        ));
    }

    #[test]
    fn dry_run_missing_vm_still_fails() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown:\n  - echo hi\n");
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let mut out = Vec::new();
        let host = FakeHostRunner::default();
        let err = run(&opts, dir.path(), &client, &host, true, &mut out)
            .unwrap_err()
            .to_string();
        let printed = String::from_utf8(out).unwrap();
        assert!(err.contains("does not exist"), "{err}");
        assert!(!printed.contains("sprite destroy"), "{printed}");
        assert!(!err.contains("sprite destroy"), "{err}");
        assert!(!client.destroyed());
        assert!(client.exec_commands().is_empty());
    }

    #[test]
    fn dry_run_missing_cli_still_fails() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown:\n  - echo hi\n");
        let client = FakeSpriteClient::default();
        let host = FakeHostRunner::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let err = run_teardown_host(dir.path(), opts, &client, &host, false)
            .unwrap_err()
            .to_string();
        assert!(err.to_lowercase().contains("install"), "{err}");
        assert!(err.contains("sprite"), "{err}");
        assert!(client.calls().is_empty());
        assert!(!client.destroyed());
        assert!(host.calls().is_empty());
    }

    #[test]
    fn dry_run_missing_name_still_fails() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "teardown: []\n");
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let err = run_teardown(dir.path(), opts, &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("name") && err.contains("--sprite"), "{err}");
        assert!(client.calls().is_empty());
        assert!(!client.destroyed());
    }

    #[test]
    fn dry_run_missing_config_still_fails() {
        let dir = TempDir::new().unwrap();
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        opts.config = Some(dir.path().join("missing.yaml"));
        let err = run_teardown(dir.path(), opts, &client, true)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("not found") || err.contains("missing.yaml"),
            "{err}"
        );
        assert!(client.calls().is_empty());
        assert!(!client.destroyed());
    }

    #[test]
    fn missing_vm_with_no_destroy() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown:\n  - echo hi\n");
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.no_destroy = true;
        let err = run_teardown(dir.path(), opts, &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not exist"), "{err}");
        assert!(client.exec_commands().is_empty());
        assert!(!client.destroyed());
    }

    #[test]
    fn middle_command_fails() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nteardown:\n  - echo one\n  - echo two\n  - echo three\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]).fail_exec_at(1, "nope");
        let err = run_teardown(dir.path(), opts_for(dir.path()), &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("2 of 3"), "{err}");
        assert!(err.contains("teardown"), "{err}");
        assert_eq!(
            client.exec_commands(),
            vec!["echo one".to_string(), "echo two".to_string()]
        );
        assert!(!client.destroyed());
    }

    #[test]
    fn destroy_failure_after_steps() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]).destroy_fails("boom");
        let err = run_teardown(dir.path(), opts_for(dir.path()), &client, true).unwrap_err();
        let chained = format!("{err:#}");
        assert!(
            chained.contains("boom") || chained.contains("destroy"),
            "{chained}"
        );
        assert_eq!(client.exec_commands(), vec!["echo hi".to_string()]);
        assert_eq!(destroy_calls(&client), 1);
    }

    #[test]
    fn empty_teardown_on_existing_vm() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let printed = run_teardown(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(client.exec_commands().is_empty());
        assert!(client.destroyed());
        assert!(!client.created());
        assert!(matches!(
            client.calls().as_slice(),
            [SpriteCall::List { .. }, SpriteCall::Destroy { .. }]
        ));
        assert!(printed.contains("destroyed"), "{printed}");
    }

    #[test]
    fn existing_vm_with_no_destroy() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nteardown:\n  - host: echo cleanup {{sprite}}\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let host = FakeHostRunner::default();
        let mut opts = opts_for(dir.path());
        opts.no_destroy = true;
        run_teardown_host(dir.path(), opts, &client, &host, true).unwrap();
        assert_eq!(host.calls().len(), 1);
        assert_eq!(host.calls()[0].command, "echo cleanup demo");
        assert!(client.exec_commands().is_empty());
        assert!(!client.destroyed());
    }

    #[test]
    fn empty_teardown_with_no_destroy() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown: []\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.no_destroy = true;
        let printed = run_teardown(dir.path(), opts, &client, true).unwrap();
        assert!(client.exec_commands().is_empty());
        assert!(!client.destroyed());
        assert!(matches!(
            client.calls().as_slice(),
            [SpriteCall::List { .. }]
        ));
        assert!(printed.contains("kept"), "{printed}");
    }

    #[test]
    fn dry_run_with_existing_vm() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nteardown:\n  - echo in-vm\n  - host: echo on-laptop\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let host = FakeHostRunner::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let printed = run_teardown_host(dir.path(), opts, &client, &host, true).unwrap();
        let exec_line = format_command(&exec_argv("demo", None, "echo in-vm"));
        let destroy_line = format_command(&destroy_argv("demo", None));
        assert!(printed.contains(&exec_line), "{printed}");
        assert!(printed.contains("host: echo on-laptop"), "{printed}");
        assert!(printed.contains(&destroy_line), "{printed}");
        assert!(printed.contains("--force"), "{printed}");
        assert!(!printed.split_whitespace().any(|w| w == "-f"), "{printed}");
        let exec_pos = printed.find(&exec_line).unwrap();
        let host_pos = printed.find("host: echo on-laptop").unwrap();
        let destroy_pos = printed.find(&destroy_line).unwrap();
        assert!(exec_pos < host_pos && host_pos < destroy_pos, "{printed}");
        assert!(client.exec_commands().is_empty());
        assert!(host.calls().is_empty());
        assert!(!client.destroyed());
        assert!(!client.created());
        assert!(printed.contains("would be destroyed"), "{printed}");
    }

    #[test]
    fn dry_run_with_no_destroy() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        opts.no_destroy = true;
        let printed = run_teardown(dir.path(), opts, &client, true).unwrap();
        assert!(
            printed.contains(&format_command(&exec_argv("demo", None, "echo hi"))),
            "{printed}"
        );
        assert!(!printed.contains("sprite destroy"), "{printed}");
        assert!(!client.destroyed());
        assert!(client.exec_commands().is_empty());
        assert!(printed.contains("would be kept"), "{printed}");
    }

    #[test]
    fn success_summary_destroyed() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let printed = run_teardown(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(printed.contains("demo"), "{printed}");
        assert!(printed.contains("destroyed"), "{printed}");
        assert!(printed.contains("1 command"), "{printed}");
        assert!(
            printed.split_whitespace().any(|word| {
                word.ends_with('s') && word.trim_end_matches('s').parse::<f64>().is_ok()
            }),
            "{printed}"
        );
        assert!(printed.contains("Teardown complete"), "{printed}");
    }

    #[test]
    fn success_summary_kept() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.no_destroy = true;
        let printed = run_teardown(dir.path(), opts, &client, true).unwrap();
        assert!(printed.contains("demo"), "{printed}");
        assert!(printed.contains("kept"), "{printed}");
        assert!(!printed.contains("destroyed"), "{printed}");
    }

    #[test]
    fn dry_run_shows_expanded_teardown_host_command() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nteardown:\n  - host: echo cleanup {{sprite}}\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let host = FakeHostRunner::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let printed = run_teardown_host(dir.path(), opts, &client, &host, true).unwrap();
        assert!(printed.contains("host: echo cleanup demo"), "{printed}");
        assert!(!printed.contains("{{sprite}}"), "{printed}");
        assert!(host.calls().is_empty());
    }

    #[test]
    fn teardown_line_needs_remote_after_earlier_step() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nteardown:\n  - echo one\n  - git clone {{remote}}\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.git = Some(GitInfo {
            branch: Some("feat".to_string()),
            commit: Some("abc".to_string()),
            remote: None,
        });
        let err = run_teardown(dir.path(), opts, &client, true).unwrap_err();
        let chained = format!("{err:#}");
        assert!(chained.contains("remote"), "{chained}");
        assert!(
            chained.contains("2 of 3") || chained.contains("2 of 2"),
            "{chained}"
        );
        assert_eq!(client.exec_commands(), vec!["echo one".to_string()]);
        assert!(!client.destroyed());
    }

    #[test]
    fn verbose_teardown_output() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\norg: acme\nteardown:\n  - echo hi\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.verbose = true;
        opts.git = Some(git_info(
            "feature/add-dashboard",
            "abc123def",
            "git@github.com:example-org/example-app.git",
        ));
        let printed = run_teardown(dir.path(), opts, &client, true).unwrap();
        assert!(
            printed.contains(&dir.path().join(".spry.yaml").display().to_string()),
            "{printed}"
        );
        assert!(printed.contains("name: demo"), "{printed}");
        assert!(printed.contains("org: acme"), "{printed}");
        assert!(
            printed.contains("branch: feature/add-dashboard"),
            "{printed}"
        );
        assert!(
            printed.contains("branch_slug: feature-add-dashboard"),
            "{printed}"
        );
        assert!(printed.contains("commit: abc123def"), "{printed}");
        assert!(
            printed.contains("remote: git@github.com:example-org/example-app.git"),
            "{printed}"
        );
        assert!(
            printed.contains(&format_command(&list_argv(Some("acme")))),
            "{printed}"
        );
        assert!(
            printed.contains(&format_command(&exec_argv("demo", Some("acme"), "echo hi"))),
            "{printed}"
        );
        assert!(
            printed.contains(&format_command(&destroy_argv("demo", Some("acme")))),
            "{printed}"
        );
        assert!(client.destroyed());
    }

    #[test]
    fn verbose_org_absent() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown: []\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.verbose = true;
        let printed = run_teardown(dir.path(), opts, &client, true).unwrap();
        assert!(printed.contains("org: absent"), "{printed}");
        assert!(!printed.contains("org: acme"), "{printed}");
        assert!(
            printed.contains(&format_command(&destroy_argv("demo", None))),
            "{printed}"
        );
    }

    #[test]
    fn host_step_fails_does_not_destroy() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nteardown:\n  - echo one\n  - host: echo cleanup\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let host = FakeHostRunner::fail("boom");
        let err = run_teardown_host(dir.path(), opts_for(dir.path()), &client, &host, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("2 of 2"), "{err}");
        assert!(err.contains("teardown"), "{err}");
        assert_eq!(client.exec_commands(), vec!["echo one".to_string()]);
        assert_eq!(host.calls().len(), 1);
        assert!(!client.destroyed());
    }

    #[test]
    fn list_failure_stops_teardown() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown:\n  - echo hi\n");
        let client = FakeSpriteClient::list_fails("list exploded");
        let err = run_teardown(dir.path(), opts_for(dir.path()), &client, true).unwrap_err();
        let chained = format!("{err:#}");
        assert!(chained.contains("list exploded"), "{chained}");
        assert!(client.exec_commands().is_empty());
        assert!(!client.destroyed());
    }

    #[test]
    fn slash_in_name_does_not_destroy() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo/feat\nteardown:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo/feat"]);
        let err = run_teardown(dir.path(), opts_for(dir.path()), &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains('/'), "{err}");
        assert!(client.calls().is_empty());
        assert!(!client.destroyed());
    }

    #[test]
    fn flag_overrides_name_and_org() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: from-file\norg: file-org\nteardown:\n  - echo hi\n",
        );
        let client = FakeSpriteClient::with_sprites(&["from-flag"]);
        let mut opts = opts_for(dir.path());
        opts.sprite = Some("from-flag".to_string());
        opts.org = Some("flag-org".to_string());
        run_teardown(dir.path(), opts, &client, true).unwrap();
        match client
            .calls()
            .into_iter()
            .find(|c| matches!(c, SpriteCall::Destroy { .. }))
        {
            Some(SpriteCall::Destroy { name, org }) => {
                assert_eq!(name, "from-flag");
                assert_eq!(org.as_deref(), Some("flag-org"));
            }
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(client.exec_commands(), vec!["echo hi".to_string()]);
    }

    #[test]
    fn verbose_with_no_destroy_omits_destroy() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nteardown:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.verbose = true;
        opts.no_destroy = true;
        let printed = run_teardown(dir.path(), opts, &client, true).unwrap();
        assert!(printed.contains("name: demo"), "{printed}");
        assert!(!printed.contains("sprite destroy"), "{printed}");
        assert!(!client.destroyed());
        assert!(printed.contains("kept"), "{printed}");
    }

    #[test]
    fn empty_teardown_dry_run_prints_destroy() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let printed = run_teardown(dir.path(), opts, &client, true).unwrap();
        assert!(
            printed.contains(&format_command(&destroy_argv("demo", None))),
            "{printed}"
        );
        assert!(printed.contains("--force"), "{printed}");
        assert!(client.exec_commands().is_empty());
        assert!(!client.destroyed());
        assert!(printed.contains("would be destroyed"), "{printed}");
    }
}
