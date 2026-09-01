use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};

use crate::config;
use crate::host::HostRunner;
use crate::sprite::{
    create_argv, format_command, list_argv, missing_cli_error, name_present, SpriteClient,
};
use crate::steps;
use crate::template::{self, GitInfo};
use crate::user_error;

#[derive(Debug, Clone, Default)]
pub struct SetupOpts {
    pub sprite: Option<String>,
    pub org: Option<String>,
    pub no_create: bool,
    pub dry_run: bool,
    pub config: Option<PathBuf>,
    pub verbose: bool,
    pub branch: Option<String>,
    /// When set, config discovery does not walk above this directory (test seam).
    pub(crate) search_root: Option<PathBuf>,
    /// When set, skip `git_info(cwd)` (test seam).
    pub(crate) git: Option<GitInfo>,
}

/// Load config, ensure the VM exists, and run setup commands.
pub fn run(
    opts: &SetupOpts,
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
    let exists = name_present(&names, &name);

    let created = ensure_vm(opts, client, &name, org, exists, out)?;
    steps::run_steps(
        &resolved.setup,
        opts.dry_run,
        opts.verbose,
        "setup",
        client,
        host,
        &name,
        org,
        &git,
        out,
    )?;

    let elapsed = started.elapsed().as_secs_f64();
    let status = match (opts.dry_run, created) {
        (true, true) => "would be created",
        (_, true) => "created",
        (_, false) => "already existed",
    };
    let n = resolved.setup.len();
    if opts.dry_run {
        writeln!(
            out,
            "Dry run complete for '{name}' ({status}) — {n} command(s) in {elapsed:.2}s"
        )?;
    } else {
        writeln!(
            out,
            "Setup complete for '{name}' ({status}) — {n} command(s) in {elapsed:.2}s"
        )?;
    }
    Ok(())
}

fn ensure_vm(
    opts: &SetupOpts,
    client: &dyn SpriteClient,
    name: &str,
    org: Option<&str>,
    exists: bool,
    out: &mut dyn Write,
) -> Result<bool> {
    if exists {
        return Ok(false);
    }

    let create_line = format_command(&create_argv(name, org));
    if opts.no_create {
        return Err(user_error(
            format!("Sprite `{name}` does not exist."),
            format!("run `{create_line}` or omit `--no-create`"),
        ));
    }

    if opts.dry_run {
        writeln!(out, "{create_line}")?;
        return Ok(true);
    }

    if opts.verbose {
        writeln!(out, "{create_line}")?;
    }
    out.flush()?;
    client
        .create(name, org)
        .with_context(|| format!("failed to create sprite `{name}`"))?;
    Ok(true)
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

    fn opts_for(dir: &Path) -> SetupOpts {
        SetupOpts {
            search_root: Some(dir.to_path_buf()),
            git: Some(GitInfo::default()),
            ..SetupOpts::default()
        }
    }

    fn git_info(branch: &str, commit: &str, remote: &str) -> GitInfo {
        GitInfo {
            branch: Some(branch.to_string()),
            commit: Some(commit.to_string()),
            remote: Some(remote.to_string()),
        }
    }

    fn run_setup(
        dir: &Path,
        opts: SetupOpts,
        client: &FakeSpriteClient,
        sprite_available: bool,
    ) -> Result<String> {
        let mut out = Vec::new();
        let host = FakeHostRunner::default();
        run(&opts, dir, client, &host, sprite_available, &mut out)?;
        Ok(String::from_utf8(out).unwrap())
    }

    fn run_setup_host(
        dir: &Path,
        opts: SetupOpts,
        client: &FakeSpriteClient,
        host: &FakeHostRunner,
        sprite_available: bool,
    ) -> Result<String> {
        let mut out = Vec::new();
        run(&opts, dir, client, host, sprite_available, &mut out)?;
        Ok(String::from_utf8(out).unwrap())
    }

    #[test]
    fn sprite_cli_missing() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::default();
        let err = run_setup(dir.path(), opts_for(dir.path()), &client, false)
            .unwrap_err()
            .to_string();
        assert!(err.to_lowercase().contains("install"), "{err}");
        assert!(err.contains("sprite"), "{err}");
        assert!(err.contains("To fix this"), "{err}");
        assert!(client.calls().is_empty());
    }

    #[test]
    fn sprite_cli_present() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup: []\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(matches!(
            client.calls().first(),
            Some(SpriteCall::List { .. })
        ));
    }

    #[test]
    fn name_missing() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "setup: []\n");
        let client = FakeSpriteClient::default();
        let err = run_setup(dir.path(), opts_for(dir.path()), &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("name") && err.contains("--sprite"), "{err}");
        assert!(err.contains("To fix this"), "{err}");
        assert!(client.calls().is_empty());
    }

    #[test]
    fn org_omitted_from_invocations() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::default();
        run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        for call in client.calls() {
            match call {
                SpriteCall::List { org }
                | SpriteCall::Create { org, .. }
                | SpriteCall::Exec { org, .. } => {
                    assert_eq!(org, None);
                }
            }
        }
        let create = client
            .calls()
            .into_iter()
            .find(|c| matches!(c, SpriteCall::Create { .. }))
            .unwrap();
        if let SpriteCall::Create { name, org } = create {
            assert_eq!(name, "demo");
            assert_eq!(org, None);
            assert_eq!(
                create_argv(&name, org.as_deref()),
                vec!["create", "demo", "--skip-console"]
            );
        }
    }

    #[test]
    fn missing_vm_is_created() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\norg: acme\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::default();
        let printed = run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(client.created());
        let calls = client.calls();
        assert!(matches!(
            calls.as_slice(),
            [
                SpriteCall::List { org: Some(_) },
                SpriteCall::Create { .. },
                SpriteCall::Exec { command, .. }
            ] if command == "echo hi"
        ));
        let SpriteCall::Create { name, org } = &calls[1] else {
            panic!("expected create");
        };
        let argv = create_argv(name, org.as_deref());
        assert!(argv.contains(&"--skip-console".to_string()));
        assert_eq!(argv, vec!["create", "demo", "--skip-console", "-o", "acme"]);
        assert!(printed.contains("created"), "{printed}");
    }

    #[test]
    fn existing_vm_is_not_created() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(!client.created());
        assert_eq!(client.exec_commands(), vec!["echo hi".to_string()]);
    }

    #[test]
    fn create_failure_stops_setup() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::create_fails("boom");
        let err = run_setup(dir.path(), opts_for(dir.path()), &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("boom") || err.contains("create"), "{err}");
        assert!(client.exec_commands().is_empty());
    }

    #[test]
    fn list_failure_stops_setup() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::list_fails("list exploded");
        let err = run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap_err();
        let chained = format!("{err:#}");
        assert!(chained.contains("list exploded"), "{chained}");
        assert!(chained.contains("list"), "{chained}");
        assert!(!client.created());
        assert!(client.exec_commands().is_empty());
    }

    #[test]
    fn missing_vm_with_no_create() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.no_create = true;
        let err = run_setup(dir.path(), opts, &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("sprite create"), "{err}");
        assert!(err.contains("--skip-console"), "{err}");
        assert!(err.contains("To fix this"), "{err}");
        assert!(!client.created());
        assert!(client.exec_commands().is_empty());
    }

    #[test]
    fn existing_vm_with_no_create() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.no_create = true;
        run_setup(dir.path(), opts, &client, true).unwrap();
        assert!(!client.created());
        assert_eq!(client.exec_commands(), vec!["echo hi".to_string()]);
    }

    #[test]
    fn dry_run_with_missing_vm() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nsetup:\n  - echo one\n  - echo two\n",
        );
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let printed = run_setup(dir.path(), opts, &client, true).unwrap();
        assert!(
            printed.contains(&format_command(&create_argv("demo", None))),
            "{printed}"
        );
        assert!(printed.contains("--skip-console"), "{printed}");
        assert!(
            printed.contains(&format_command(&exec_argv("demo", None, "echo one"))),
            "{printed}"
        );
        assert!(
            printed.contains(&format_command(&exec_argv("demo", None, "echo two"))),
            "{printed}"
        );
        let create_pos = printed.find("sprite create").unwrap();
        let exec_pos = printed.find("sprite exec").unwrap();
        assert!(create_pos < exec_pos, "{printed}");
        assert!(!client.created());
        assert!(client.exec_commands().is_empty());
        assert!(matches!(
            client.calls().as_slice(),
            [SpriteCall::List { .. }]
        ));
    }

    #[test]
    fn dry_run_with_existing_vm() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let printed = run_setup(dir.path(), opts, &client, true).unwrap();
        assert!(!printed.contains("sprite create"), "{printed}");
        assert!(
            printed.contains(&format_command(&exec_argv("demo", None, "echo hi"))),
            "{printed}"
        );
        assert!(!client.created());
        assert!(client.exec_commands().is_empty());
    }

    #[test]
    fn dry_run_still_enforces_no_create() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        opts.no_create = true;
        let err = run_setup(dir.path(), opts, &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("sprite create"), "{err}");
        assert!(!client.created());
        assert!(client.exec_commands().is_empty());
    }

    #[test]
    fn all_commands_succeed() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nsetup:\n  - echo one\n  - echo two\n  - echo three\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let printed = run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(printed.contains("[1/3]"), "{printed}");
        assert!(printed.contains("[2/3]"), "{printed}");
        assert!(printed.contains("[3/3]"), "{printed}");
        assert_eq!(
            client.exec_commands(),
            vec![
                "echo one".to_string(),
                "echo two".to_string(),
                "echo three".to_string()
            ]
        );
    }

    #[test]
    fn middle_command_fails() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nsetup:\n  - echo one\n  - echo two\n  - echo three\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]).fail_exec_at(1, "nope");
        let err = run_setup(dir.path(), opts_for(dir.path()), &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("2 of 3"), "{err}");
        assert!(err.contains("setup"), "{err}");
        assert_eq!(
            client.exec_commands(),
            vec!["echo one".to_string(), "echo two".to_string()]
        );
    }

    #[test]
    fn exec_includes_org_when_set() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\norg: acme\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        match client.calls().last() {
            Some(SpriteCall::Exec { name, org, command }) => {
                assert_eq!(name, "demo");
                assert_eq!(org.as_deref(), Some("acme"));
                assert_eq!(command, "echo hi");
                assert_eq!(
                    exec_argv(name, org.as_deref(), command),
                    vec!["exec", "-s", "demo", "-o", "acme", "--", "sh", "-c", "echo hi"]
                );
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn success_summary() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nsetup:\n  - echo one\n  - echo two\n",
        );
        let client = FakeSpriteClient::default();
        let printed = run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(printed.contains("demo"), "{printed}");
        assert!(printed.contains("created"), "{printed}");
        assert!(printed.contains("2 command"), "{printed}");
        assert!(
            printed.split_whitespace().any(|word| {
                word.ends_with('s') && word.trim_end_matches('s').parse::<f64>().is_ok()
            }),
            "{printed}"
        );
        assert!(printed.contains("Setup complete"), "{printed}");
    }

    #[test]
    fn verbose_setup_output() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\norg: acme\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.verbose = true;
        let printed = run_setup(dir.path(), opts, &client, true).unwrap();
        assert!(
            printed.contains(&dir.path().join(".spry.yaml").display().to_string()),
            "{printed}"
        );
        assert!(printed.contains("name: demo"), "{printed}");
        assert!(printed.contains("org: acme"), "{printed}");
        assert!(
            printed.contains(&format_command(&list_argv(Some("acme")))),
            "{printed}"
        );
        assert!(
            printed.contains(&format_command(&create_argv("demo", Some("acme")))),
            "{printed}"
        );
        assert!(
            printed.contains(&format_command(&exec_argv("demo", Some("acme"), "echo hi"))),
            "{printed}"
        );
    }

    #[test]
    fn verbose_org_absent() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup: []\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.verbose = true;
        let printed = run_setup(dir.path(), opts, &client, true).unwrap();
        assert!(printed.contains("org: absent"), "{printed}");
        assert!(!printed.contains("org: acme"), "{printed}");
    }

    #[test]
    fn empty_setup_creates_missing_vm() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup: []\n");
        let client = FakeSpriteClient::default();
        let printed = run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(client.created());
        assert!(client.exec_commands().is_empty());
        assert!(printed.contains("created"), "{printed}");
        assert!(printed.contains("0 command"), "{printed}");
    }

    #[test]
    fn empty_setup_on_existing_vm() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let printed = run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(!client.created());
        assert!(client.exec_commands().is_empty());
        assert!(printed.contains("already existed"), "{printed}");
    }

    #[test]
    fn setup_outside_git() {
        let dir = TempDir::new().unwrap();
        assert!(!dir.path().join(".git").exists());
        write_recipe(dir.path(), "name: demo\nsetup: []\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(matches!(
            client.calls().first(),
            Some(SpriteCall::List { .. })
        ));
    }

    #[test]
    fn dry_run_still_enforces_missing_cli() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let err = run_setup(dir.path(), opts, &client, false)
            .unwrap_err()
            .to_string();
        assert!(err.to_lowercase().contains("install"), "{err}");
        assert!(client.calls().is_empty());
    }

    #[test]
    fn dry_run_still_enforces_missing_config() {
        let dir = TempDir::new().unwrap();
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let err = run_setup(dir.path(), opts, &client, true)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("spry init") || err.contains("--config"),
            "{err}"
        );
        assert!(client.calls().is_empty());
    }

    #[test]
    fn dry_run_still_enforces_missing_name() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "setup: []\n");
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let err = run_setup(dir.path(), opts, &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("name") && err.contains("--sprite"), "{err}");
        assert!(client.calls().is_empty());
    }

    #[test]
    fn missing_config_has_hint() {
        let dir = TempDir::new().unwrap();
        let client = FakeSpriteClient::default();
        let err = run_setup(dir.path(), opts_for(dir.path()), &client, true)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("spry init") || err.contains("--config"),
            "{err}"
        );
        assert!(err.contains("To fix this"), "{err}");
        assert!(client.calls().is_empty());
    }

    #[test]
    fn shows_exec_output() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]).exec_output("hello from vm\n");
        let printed = run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(printed.contains("hello from vm"), "{printed}");
    }

    #[test]
    fn shows_exec_stderr() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - echo hi\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]).exec_streams("out\n", "err\n");
        let printed = run_setup(dir.path(), opts_for(dir.path()), &client, true).unwrap();
        assert!(printed.contains("out"), "{printed}");
        assert!(printed.contains("err"), "{printed}");
    }

    #[test]
    fn empty_setup_missing_vm_with_no_create() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup: []\n");
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.no_create = true;
        let err = run_setup(dir.path(), opts, &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("sprite create"), "{err}");
        assert!(!client.created());
        assert!(client.exec_commands().is_empty());
    }

    #[test]
    fn setup_flag_overrides_name_and_org() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: from-file\norg: file-org\nsetup:\n  - echo hi\n",
        );
        let client = FakeSpriteClient::with_sprites(&["from-flag"]);
        let mut opts = opts_for(dir.path());
        opts.sprite = Some("from-flag".to_string());
        opts.org = Some("flag-org".to_string());
        run_setup(dir.path(), opts, &client, true).unwrap();
        match client.calls().last() {
            Some(SpriteCall::Exec { name, org, .. }) => {
                assert_eq!(name, "from-flag");
                assert_eq!(org.as_deref(), Some("flag-org"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn unique_vm_name_per_branch() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: myapp-{{branch_slug}}\nsetup:\n  - echo hi\n",
        );
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.git = Some(git_info(
            "feature/add-dashboard",
            "abc123def",
            "git@github.com:example-org/example-app.git",
        ));
        run_setup(dir.path(), opts, &client, true).unwrap();
        match client
            .calls()
            .iter()
            .find(|c| matches!(c, SpriteCall::Create { .. }))
        {
            Some(SpriteCall::Create { name, .. }) => {
                assert_eq!(name, "myapp-feature-add-dashboard");
            }
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(client.exec_commands(), vec!["echo hi".to_string()]);
    }

    #[test]
    fn sprite_flag_is_expanded() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: from-file\nsetup: []\n");
        let client = FakeSpriteClient::with_sprites(&["app-feat-x"]);
        let mut opts = opts_for(dir.path());
        opts.sprite = Some("app-{{branch_slug}}".to_string());
        opts.branch = Some("feat/x".to_string());
        run_setup(dir.path(), opts, &client, true).unwrap();
        match client.calls().first() {
            Some(SpriteCall::List { .. }) => {}
            other => panic!("unexpected {other:?}"),
        }
        assert!(!client.created());
    }

    #[test]
    fn branch_flag_without_git() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: app-{{branch_slug}}\nsetup:\n  - git checkout \"{{branch}}\"\n",
        );
        let client = FakeSpriteClient::with_sprites(&["app-feat-x"]);
        let mut opts = opts_for(dir.path());
        opts.branch = Some("feat/x".to_string());
        run_setup(dir.path(), opts, &client, true).unwrap();
        assert_eq!(
            client.exec_commands(),
            vec!["git checkout \"feat/x\"".to_string()]
        );
    }

    #[test]
    fn dry_run_shows_expanded_checkout() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nsetup:\n  - git checkout \"{{branch}}\"\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        opts.branch = Some("feat/x".to_string());
        let printed = run_setup(dir.path(), opts, &client, true).unwrap();
        assert!(printed.contains("feat/x"), "{printed}");
        assert!(!printed.contains("{{branch}}"), "{printed}");
        assert!(client.exec_commands().is_empty());
    }

    #[test]
    fn checkout_uses_real_branch_not_slug() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nsetup:\n  - git checkout \"{{branch}}\"\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.git = Some(git_info(
            "feature/add-dashboard",
            "abc123def",
            "git@github.com:example-org/example-app.git",
        ));
        run_setup(dir.path(), opts, &client, true).unwrap();
        assert_eq!(
            client.exec_commands(),
            vec!["git checkout \"feature/add-dashboard\"".to_string()]
        );
    }

    #[test]
    fn name_needs_branch_and_git_is_missing() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: app-{{branch_slug}}\nsetup: []\n");
        let client = FakeSpriteClient::default();
        let err = run_setup(dir.path(), opts_for(dir.path()), &client, true)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("branch_slug") || err.contains("--branch"),
            "{err}"
        );
        assert!(err.contains("To fix this"), "{err}");
        assert!(client.calls().is_empty());
    }

    #[test]
    fn setup_line_needs_remote_after_provision() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\nsetup:\n  - echo one\n  - git clone {{remote}}\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.git = Some(GitInfo {
            branch: Some("feat".to_string()),
            commit: Some("abc".to_string()),
            remote: None,
        });
        let err = run_setup(dir.path(), opts, &client, true).unwrap_err();
        let chained = format!("{err:#}");
        assert!(chained.contains("remote"), "{chained}");
        assert!(
            chained.contains("2 of 3") || chained.contains("2 of 2"),
            "{chained}"
        );
        assert_eq!(client.exec_commands(), vec!["echo one".to_string()]);
    }

    #[test]
    fn name_with_slash_from_raw_branch_fails() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: \"{{branch}}\"\nsetup: []\n");
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.branch = Some("feature/add-dashboard".to_string());
        let err = run_setup(dir.path(), opts, &client, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("{{branch_slug}}"), "{err}");
        assert!(client.calls().is_empty());
    }

    #[test]
    fn verbose_prints_template_context() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup: []\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let mut opts = opts_for(dir.path());
        opts.verbose = true;
        opts.git = Some(git_info(
            "feature/add-dashboard",
            "abc123def",
            "git@github.com:example-org/example-app.git",
        ));
        let printed = run_setup(dir.path(), opts, &client, true).unwrap();
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
    }

    #[test]
    fn host_setup_runs_on_host_with_sprite_env() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\norg: acme\nsetup:\n  - echo in-vm\n  - host: gh repo deploy-key add\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let host = FakeHostRunner::default();
        run_setup_host(dir.path(), opts_for(dir.path()), &client, &host, true).unwrap();
        assert_eq!(client.exec_commands(), vec!["echo in-vm".to_string()]);
        let calls = host.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].command, "gh repo deploy-key add");
        assert!(
            calls[0]
                .env
                .iter()
                .any(|(k, v)| k == "SPRITE" && v == "demo"),
            "{:?}",
            calls[0].env
        );
        assert!(
            calls[0].env.iter().any(|(k, v)| k == "ORG" && v == "acme"),
            "{:?}",
            calls[0].env
        );
    }

    #[test]
    fn dry_run_prints_host_prefix_not_sprite_exec() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup:\n  - host: echo on-laptop\n");
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let host = FakeHostRunner::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let printed = run_setup_host(dir.path(), opts, &client, &host, true).unwrap();
        assert!(printed.contains("host: echo on-laptop"), "{printed}");
        assert!(!printed.contains("sprite exec"), "{printed}");
        assert!(host.calls().is_empty());
        assert!(client.exec_commands().is_empty());
    }

    #[test]
    fn host_command_expands_sprite_placeholder() {
        let dir = TempDir::new().unwrap();
        write_recipe(
            dir.path(),
            "name: demo\norg: acme\nsetup:\n  - host: sprite exec -s {{sprite}} -o {{org}} -- true\n",
        );
        let client = FakeSpriteClient::with_sprites(&["demo"]);
        let host = FakeHostRunner::default();
        run_setup_host(dir.path(), opts_for(dir.path()), &client, &host, true).unwrap();
        assert_eq!(
            host.calls()[0].command,
            "sprite exec -s demo -o acme -- true"
        );
    }

    #[test]
    fn dry_run_missing_vm_summary_says_would_be_created() {
        let dir = TempDir::new().unwrap();
        write_recipe(dir.path(), "name: demo\nsetup: []\n");
        let client = FakeSpriteClient::default();
        let mut opts = opts_for(dir.path());
        opts.dry_run = true;
        let printed = run_setup(dir.path(), opts, &client, true).unwrap();
        assert!(printed.contains("would be created"), "{printed}");
        assert!(!printed.contains("Setup complete"), "{printed}");
        assert!(!client.created());
    }
}
