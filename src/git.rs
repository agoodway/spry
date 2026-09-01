use std::path::Path;
use std::process::Command;

use crate::template::GitInfo;

/// Read branch, commit, and origin URL from `cwd`. Missing git or a failed
/// command leaves that field absent. `HEAD` from `--abbrev-ref` is treated as
/// no branch (detached).
pub fn git_info(cwd: &Path) -> GitInfo {
    GitInfo {
        branch: git_stdout(cwd, &["rev-parse", "--abbrev-ref", "HEAD"])
            .filter(|name| name.as_str() != "HEAD"),
        commit: git_stdout(cwd, &["rev-parse", "HEAD"]),
        remote: git_stdout(cwd, &["remote", "get-url", "origin"]),
    }
}

fn git_stdout(cwd: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn git(dir: &Path, args: &[&str]) -> std::process::Output {
        Command::new("git")
            .current_dir(dir)
            .args(args)
            .output()
            .expect("git")
    }

    fn require_git() {
        let ok = Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        assert!(ok, "git is required for these tests");
    }

    fn init_repo(dir: &Path) {
        require_git();
        let status = git(dir, &["init", "-b", "main"]).status;
        assert!(status.success(), "git init");
        git(dir, &["config", "user.email", "spry@example.test"]);
        git(dir, &["config", "user.name", "Spry"]);
        git(dir, &["config", "commit.gpgsign", "false"]);
        fs::write(dir.join("README"), "hi").unwrap();
        assert!(git(dir, &["add", "README"]).status.success());
        assert!(git(dir, &["commit", "-m", "init"]).status.success());
    }

    #[test]
    fn reads_branch_commit_and_origin() {
        let dir = TempDir::new().unwrap();
        init_repo(dir.path());
        assert!(git(
            dir.path(),
            &[
                "remote",
                "add",
                "origin",
                "git@github.com:example-org/example-app.git"
            ]
        )
        .status
        .success());
        assert!(git(
            dir.path(),
            &["checkout", "-b", "feature/add-dashboard"]
        )
        .status
        .success());

        let info = git_info(dir.path());
        assert_eq!(
            info.branch.as_deref(),
            Some("feature/add-dashboard")
        );
        assert!(
            info.commit.as_ref().is_some_and(|s| s.len() >= 7),
            "{:?}",
            info.commit
        );
        assert_eq!(
            info.remote.as_deref(),
            Some("git@github.com:example-org/example-app.git")
        );
    }

    #[test]
    fn non_repo_is_absent() {
        let dir = TempDir::new().unwrap();
        let info = git_info(dir.path());
        assert_eq!(info, GitInfo::default());
    }

    #[test]
    fn detached_head_has_commit_not_branch() {
        let dir = TempDir::new().unwrap();
        init_repo(dir.path());
        let sha = git_stdout(dir.path(), &["rev-parse", "HEAD"]).unwrap();
        assert!(git(dir.path(), &["checkout", "--detach", "HEAD"])
            .status
            .success());
        let info = git_info(dir.path());
        assert_eq!(info.branch, None);
        assert_eq!(info.commit.as_deref(), Some(sha.as_str()));
    }

    #[test]
    fn missing_origin() {
        let dir = TempDir::new().unwrap();
        init_repo(dir.path());
        let info = git_info(dir.path());
        assert!(info.branch.is_some());
        assert!(info.commit.is_some());
        assert_eq!(info.remote, None);
    }
}
