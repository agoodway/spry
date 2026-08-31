use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::user_error;

/// Captured output from `sprite exec`. Production streaming leaves these empty.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecOutput {
    pub stdout: String,
    pub stderr: String,
}

/// Runtime wrapper around the `sprite` CLI.
pub trait SpriteClient {
    fn list(&self, org: Option<&str>) -> Result<Vec<String>>;
    fn create(&self, name: &str, org: Option<&str>) -> Result<()>;
    fn exec(&self, name: &str, org: Option<&str>, command: &str) -> Result<ExecOutput>;
}

/// Production client that shells out to a resolved `sprite` binary.
#[derive(Debug, Clone)]
pub struct CommandClient {
    sprite_bin: PathBuf,
}

impl CommandClient {
    pub fn new(sprite_bin: PathBuf) -> Self {
        Self { sprite_bin }
    }

    fn command(&self) -> Command {
        Command::new(&self.sprite_bin)
    }
}

impl SpriteClient for CommandClient {
    fn list(&self, org: Option<&str>) -> Result<Vec<String>> {
        let args = list_argv(org);
        // `Command::output()` forces stderr to a pipe. Spawn with stdout piped
        // and stderr inherited so sprite diagnostics still reach the terminal.
        let child = self
            .command()
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("failed to run `sprite list`")?;
        let output = child
            .wait_with_output()
            .context("failed to run `sprite list`")?;
        if !output.status.success() {
            bail!(
                "sprite list failed with status {}",
                output.status.code().unwrap_or(-1)
            );
        }
        Ok(parse_list_names(&String::from_utf8_lossy(&output.stdout)))
    }

    fn create(&self, name: &str, org: Option<&str>) -> Result<()> {
        let args = create_argv(name, org);
        let status = self
            .command()
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("failed to run `sprite create`")?;
        if !status.success() {
            bail!(
                "sprite create failed with status {}",
                status.code().unwrap_or(-1)
            );
        }
        Ok(())
    }

    fn exec(&self, name: &str, org: Option<&str>, command: &str) -> Result<ExecOutput> {
        let args = exec_argv(name, org, command);
        let status = self
            .command()
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("failed to run `sprite exec`")?;
        if !status.success() {
            bail!(
                "sprite exec failed with status {}",
                status.code().unwrap_or(-1)
            );
        }
        Ok(ExecOutput::default())
    }
}

pub fn list_argv(org: Option<&str>) -> Vec<String> {
    let mut args = vec!["list".to_string()];
    push_org(&mut args, org);
    args
}

pub fn create_argv(name: &str, org: Option<&str>) -> Vec<String> {
    let mut args = vec![
        "create".to_string(),
        name.to_string(),
        "--skip-console".to_string(),
    ];
    push_org(&mut args, org);
    args
}

pub fn exec_argv(name: &str, org: Option<&str>, command: &str) -> Vec<String> {
    let mut args = vec!["exec".to_string(), "-s".to_string(), name.to_string()];
    push_org(&mut args, org);
    args.push("--".to_string());
    args.push("sh".to_string());
    args.push("-c".to_string());
    args.push(command.to_string());
    args
}

fn push_org(args: &mut Vec<String>, org: Option<&str>) {
    if let Some(org) = org {
        args.push("-o".to_string());
        args.push(org.to_string());
    }
}

/// Format a complete `sprite …` command line for display.
pub fn format_command(args: &[String]) -> String {
    let mut parts = vec!["sprite".to_string()];
    for arg in args {
        parts.push(quote_arg(arg));
    }
    parts.join(" ")
}

fn is_safe_cli_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '=' | ':' | ',' | '+' | '-')
}

fn quote_arg(arg: &str) -> String {
    if !arg.is_empty() && arg.chars().all(is_safe_cli_char) {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

/// Parse VM names from `sprite list` stdout.
///
/// A name is the whole trimmed line and/or its first whitespace-separated field.
/// Header rows (for example a first field of `NAME`) are not skipped: the spec
/// matches any first field, so a table header can collide with a VM of that name.
pub fn parse_list_names(stdout: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = HashSet::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        push_unique(&mut names, &mut seen, trimmed);
        if let Some(first) = trimmed.split_whitespace().next() {
            push_unique(&mut names, &mut seen, first);
        }
    }
    names
}

fn push_unique(names: &mut Vec<String>, seen: &mut HashSet<String>, name: &str) {
    if seen.insert(name.to_string()) {
        names.push(name.to_string());
    }
}

pub fn name_present(names: &[String], target: &str) -> bool {
    names.iter().any(|name| name == target)
}

/// Look up `sprite` on PATH (LookPath / `which` equivalent).
pub fn sprite_on_path() -> bool {
    find_in_path("sprite", std::env::var_os("PATH").as_deref()).is_some()
}

pub fn find_in_path(name: &str, path_env: Option<&OsStr>) -> Option<PathBuf> {
    let path_env = path_env?;
    for dir in std::env::split_paths(path_env) {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let pathext =
                std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.BAT;.CMD;.COM".to_string());
            for ext in pathext.split(';').filter(|ext| !ext.is_empty()) {
                let candidate = dir.join(format!("{name}{ext}"));
                if is_executable(&candidate) {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|meta| meta.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

pub fn missing_cli_error() -> anyhow::Error {
    user_error(
        "The `sprite` CLI was not found on PATH.",
        "install the sprite CLI and ensure `sprite` is available on your PATH",
    )
}

/// Recorded `sprite` operation used by tests.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpriteCall {
    List {
        org: Option<String>,
    },
    Create {
        name: String,
        org: Option<String>,
    },
    Exec {
        name: String,
        org: Option<String>,
        command: String,
    },
}

/// Test double that records calls and returns scripted results.
#[cfg(test)]
#[derive(Debug)]
pub struct FakeSpriteClient {
    calls: std::cell::RefCell<Vec<SpriteCall>>,
    list_result: Result<Vec<String>, String>,
    create_result: Result<(), String>,
    exec_script: Vec<Result<ExecOutput, String>>,
    exec_index: std::cell::Cell<usize>,
}

#[cfg(test)]
impl Default for FakeSpriteClient {
    fn default() -> Self {
        Self {
            calls: std::cell::RefCell::new(Vec::new()),
            list_result: Ok(Vec::new()),
            create_result: Ok(()),
            exec_script: Vec::new(),
            exec_index: std::cell::Cell::new(0),
        }
    }
}

#[cfg(test)]
impl FakeSpriteClient {
    pub fn with_sprites(names: &[&str]) -> Self {
        Self {
            list_result: Ok(names.iter().map(|s| (*s).to_string()).collect()),
            ..Self::default()
        }
    }

    pub fn list_fails(message: &str) -> Self {
        Self {
            list_result: Err(message.to_string()),
            ..Self::default()
        }
    }

    pub fn create_fails(message: &str) -> Self {
        Self {
            create_result: Err(message.to_string()),
            ..Self::default()
        }
    }

    pub fn fail_exec_at(mut self, index: usize, message: &str) -> Self {
        if self.exec_script.len() <= index {
            self.exec_script
                .resize_with(index + 1, || Ok(ExecOutput::default()));
        }
        self.exec_script[index] = Err(message.to_string());
        self
    }

    pub fn exec_output(mut self, stdout: &str) -> Self {
        self.exec_script.push(Ok(ExecOutput {
            stdout: stdout.to_string(),
            stderr: String::new(),
        }));
        self
    }

    pub fn exec_streams(mut self, stdout: &str, stderr: &str) -> Self {
        self.exec_script.push(Ok(ExecOutput {
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
        }));
        self
    }

    pub fn calls(&self) -> Vec<SpriteCall> {
        self.calls.borrow().clone()
    }

    pub fn created(&self) -> bool {
        self.calls
            .borrow()
            .iter()
            .any(|call| matches!(call, SpriteCall::Create { .. }))
    }

    pub fn exec_commands(&self) -> Vec<String> {
        self.calls
            .borrow()
            .iter()
            .filter_map(|call| match call {
                SpriteCall::Exec { command, .. } => Some(command.clone()),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
impl SpriteClient for FakeSpriteClient {
    fn list(&self, org: Option<&str>) -> Result<Vec<String>> {
        self.calls.borrow_mut().push(SpriteCall::List {
            org: org.map(str::to_string),
        });
        match &self.list_result {
            Ok(names) => Ok(names.clone()),
            Err(message) => bail!("{message}"),
        }
    }

    fn create(&self, name: &str, org: Option<&str>) -> Result<()> {
        self.calls.borrow_mut().push(SpriteCall::Create {
            name: name.to_string(),
            org: org.map(str::to_string),
        });
        match &self.create_result {
            Ok(()) => Ok(()),
            Err(message) => bail!("{message}"),
        }
    }

    fn exec(&self, name: &str, org: Option<&str>, command: &str) -> Result<ExecOutput> {
        self.calls.borrow_mut().push(SpriteCall::Exec {
            name: name.to_string(),
            org: org.map(str::to_string),
            command: command.to_string(),
        });
        let index = self.exec_index.get();
        self.exec_index.set(index + 1);
        match self.exec_script.get(index) {
            Some(Err(message)) => bail!("{message}"),
            Some(Ok(output)) => Ok(output.clone()),
            None => Ok(ExecOutput::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn create_argv_includes_skip_console() {
        let args = create_argv("demo", None);
        assert_eq!(args, vec!["create", "demo", "--skip-console"]);
        assert!(!args.contains(&"-o".to_string()));
    }

    #[test]
    fn create_argv_includes_org() {
        let args = create_argv("demo", Some("acme"));
        assert!(args.contains(&"--skip-console".to_string()));
        assert_eq!(args, vec!["create", "demo", "--skip-console", "-o", "acme"]);
    }

    #[test]
    fn list_argv_omits_org_when_absent() {
        assert_eq!(list_argv(None), vec!["list"]);
    }

    #[test]
    fn list_argv_includes_org() {
        assert_eq!(list_argv(Some("acme")), vec!["list", "-o", "acme"]);
    }

    #[test]
    fn exec_argv_shape() {
        let args = exec_argv("demo", None, "echo hi");
        assert_eq!(
            args,
            vec!["exec", "-s", "demo", "--", "sh", "-c", "echo hi"]
        );
    }

    #[test]
    fn exec_argv_includes_org() {
        let args = exec_argv("demo", Some("acme"), "echo hi");
        assert_eq!(
            args,
            vec!["exec", "-s", "demo", "-o", "acme", "--", "sh", "-c", "echo hi"]
        );
    }

    #[test]
    fn parse_name_per_line() {
        let names = parse_list_names("demo\nother\n\n");
        assert!(name_present(&names, "demo"));
        assert!(name_present(&names, "other"));
        assert!(!name_present(&names, "missing"));
    }

    #[test]
    fn parse_first_field_table() {
        let names =
            parse_list_names("NAME     STATUS\ndemo     running  1h\nother    stopped  2d\n");
        assert!(name_present(&names, "demo"));
        assert!(name_present(&names, "other"));
        assert!(name_present(&names, "NAME"));
    }

    #[test]
    fn finds_executable_on_path() {
        let dir = TempDir::new().unwrap();
        let bin = dir.path().join("sprite");
        fs::write(&bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let found = find_in_path("sprite", Some(dir.path().as_os_str()));
        assert_eq!(found.as_deref(), Some(bin.as_path()));
    }

    #[test]
    fn missing_when_not_on_path() {
        let dir = TempDir::new().unwrap();
        assert!(find_in_path("sprite", Some(dir.path().as_os_str())).is_none());
    }

    #[test]
    fn missing_cli_error_mentions_install() {
        let err = missing_cli_error().to_string();
        assert!(err.to_lowercase().contains("install"), "{err}");
        assert!(err.contains("sprite"), "{err}");
        assert!(err.contains("To fix this"), "{err}");
    }

    #[test]
    fn quote_arg_quotes_shell_metacharacters() {
        assert_eq!(quote_arg("echo hi"), "'echo hi'");
        assert_eq!(quote_arg("a;b"), "'a;b'");
        assert_eq!(quote_arg("a&b"), "'a&b'");
        assert_eq!(quote_arg("demo"), "demo");
        assert_eq!(quote_arg(""), "''");
    }

    #[cfg(unix)]
    fn install_stub(dir: &Path, body: &str) -> PathBuf {
        let path = dir.join("sprite");
        fs::write(&path, body).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[cfg(unix)]
    #[test]
    fn command_client_list_create_exec_via_stub() {
        let dir = TempDir::new().unwrap();
        let script = r#"#!/bin/sh
here=$(dirname "$0")
printf '%s\n' "$@" >> "$here/argv.log"
case "$1" in
list) printf 'demo\n' ;;
create) ;;
exec) printf 'out\n'; printf 'err\n' >&2 ;;
*) exit 2 ;;
esac
"#;
        let bin = install_stub(dir.path(), script);
        let client = CommandClient::new(bin);
        let names = client.list(None).unwrap();
        assert!(name_present(&names, "demo"));
        client.create("demo", None).unwrap();
        client.exec("demo", None, "echo hi").unwrap();
        let log = fs::read_to_string(dir.path().join("argv.log")).unwrap();
        assert!(log.contains("list"), "{log}");
        assert!(log.contains("create"), "{log}");
        assert!(log.contains("--skip-console"), "{log}");
        assert!(log.contains("exec"), "{log}");
        assert!(log.contains("echo hi"), "{log}");
    }

    #[cfg(unix)]
    #[test]
    fn command_client_list_failure_status() {
        let dir = TempDir::new().unwrap();
        let bin = install_stub(dir.path(), "#!/bin/sh\necho boom >&2\nexit 3\n");
        let client = CommandClient::new(bin);
        let err = client.list(None).unwrap_err().to_string();
        assert!(err.contains("3") || err.contains("list"), "{err}");
    }

    #[cfg(unix)]
    #[test]
    fn command_client_uses_resolved_binary_not_path_search() {
        let dir = TempDir::new().unwrap();
        let bin = install_stub(dir.path(), "#!/bin/sh\necho from-stub\n");
        let client = CommandClient::new(bin);
        let names = client.list(None).unwrap();
        assert!(name_present(&names, "from-stub"));
    }
}
