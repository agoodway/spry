use anyhow::Result;

use crate::user_error;

/// Git metadata used to expand recipe placeholders.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitInfo {
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub remote: Option<String>,
}

/// Overlay `--branch` onto git-discovered info. Commit and remote stay from git.
pub fn apply_branch_override(mut info: GitInfo, branch: Option<&str>) -> GitInfo {
    if let Some(branch) = branch.filter(|s| !s.is_empty()) {
        info.branch = Some(branch.to_string());
    }
    info
}

/// Slug a branch name for use in a sprite VM name.
///
/// Each run of characters outside `[A-Za-z0-9._-]` becomes a single `-`.
/// Leading and trailing `-` are stripped. Case is preserved.
pub fn slug(branch: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for c in branch.chars() {
        if is_slug_safe(c) {
            out.push(c);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn is_slug_safe(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')
}

/// Extra values available when expanding setup commands (not recipe `name`).
#[derive(Debug, Clone, Copy, Default)]
pub struct SetupVars<'a> {
    pub sprite: Option<&'a str>,
    pub org: Option<&'a str>,
}

/// Expand git placeholders in `input`. Used for the recipe `name` field.
pub fn expand(input: &str, info: &GitInfo) -> Result<String> {
    expand_in(input, info, SetupVars::default())
}

/// Expand git placeholders plus `{{sprite}}` / `{{org}}` in a setup command.
pub fn expand_in(input: &str, info: &GitInfo, vars: SetupVars<'_>) -> Result<String> {
    let mut result = String::new();
    let mut rest = input;
    while let Some(start) = rest.find("{{") {
        result.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        let Some(end) = rest.find("}}") else {
            return Err(user_error(
                "Unclosed placeholder starting with `{{`.",
                "close it with `}}`, or remove the extra braces",
            ));
        };
        let name = &rest[..end];
        result.push_str(&resolve(name, info, vars)?);
        rest = &rest[end + 2..];
    }
    result.push_str(rest);
    Ok(result)
}

fn resolve(name: &str, info: &GitInfo, vars: SetupVars<'_>) -> Result<String> {
    match name {
        "branch" => required(info.branch.clone(), "branch"),
        "branch_slug" => {
            let branch = required(info.branch.clone(), "branch_slug")?;
            let slugged = slug(&branch);
            if slugged.is_empty() {
                Err(missing_error("branch_slug"))
            } else {
                Ok(slugged)
            }
        }
        "commit" => required(info.commit.clone(), "commit"),
        "remote" => required(info.remote.clone(), "remote"),
        "sprite" => required(vars.sprite.map(str::to_string), "sprite"),
        "org" => required(vars.org.map(str::to_string), "org"),
        other => Err(user_error(
            format!("Unknown placeholder `{{{{{other}}}}}`.",),
            "use {{branch}}, {{branch_slug}}, {{commit}}, {{remote}}, {{sprite}}, or {{org}}",
        )),
    }
}

fn required(value: Option<String>, name: &str) -> Result<String> {
    value.ok_or_else(|| missing_error(name))
}

fn missing_error(name: &str) -> anyhow::Error {
    let fix = match name {
        "branch" | "branch_slug" => {
            "run `spry setup` from a git checkout or pass `--branch <name>`"
        }
        "commit" => "run `spry setup` from a git checkout",
        "remote" => "run `spry setup` from a git checkout that has an `origin` remote",
        "sprite" => "pass `--sprite` or set `name` in the recipe",
        "org" => "set `org` in the recipe or pass `--org`",
        _ => "use {{branch}}, {{branch_slug}}, {{commit}}, {{remote}}, {{sprite}}, or {{org}}",
    };
    user_error(format!("Could not resolve {{{{{name}}}}}."), fix)
}

/// Fail if an expanded sprite name still contains `/`.
pub fn reject_slash_in_name(name: &str) -> Result<()> {
    if name.contains('/') {
        Err(user_error(
            format!("Sprite name `{name}` contains `/`."),
            "use `{{branch_slug}}` in `name` (or pass `--sprite` without slashes)",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(branch: Option<&str>, commit: Option<&str>, remote: Option<&str>) -> GitInfo {
        GitInfo {
            branch: branch.map(str::to_string),
            commit: commit.map(str::to_string),
            remote: remote.map(str::to_string),
        }
    }

    #[test]
    fn all_known_placeholders_expand() {
        let git = info(
            Some("feature/add-dashboard"),
            Some("abc123def"),
            Some("git@github.com:example-org/example-app.git"),
        );
        assert_eq!(
            expand("{{branch}}", &git).unwrap(),
            "feature/add-dashboard"
        );
        assert_eq!(
            expand("{{branch_slug}}", &git).unwrap(),
            "feature-add-dashboard"
        );
        assert_eq!(expand("{{commit}}", &git).unwrap(), "abc123def");
        assert_eq!(
            expand("{{remote}}", &git).unwrap(),
            "git@github.com:example-org/example-app.git"
        );
    }

    #[test]
    fn unknown_placeholder_fails() {
        let err = expand("{{foo}}", &GitInfo::default())
            .unwrap_err()
            .to_string();
        assert!(err.contains("foo"), "{err}");
        assert!(err.contains("To fix this"), "{err}");
    }

    #[test]
    fn interior_whitespace_is_unknown() {
        let err = expand("{{ branch }}", &info(Some("feat"), None, None))
            .unwrap_err()
            .to_string();
        assert!(err.to_lowercase().contains("unknown"), "{err}");
    }

    #[test]
    fn values_are_not_reexpanded() {
        let git = info(Some("{{remote}}"), None, Some("git@example.com:r.git"));
        assert_eq!(expand("{{branch}}", &git).unwrap(), "{{remote}}");
    }

    #[test]
    fn slashed_branch_slug() {
        assert_eq!(
            slug("feature/add-dashboard"),
            "feature-add-dashboard"
        );
    }

    #[test]
    fn adjacent_separators_collapse() {
        assert_eq!(slug("feat//x y"), "feat-x-y");
    }

    #[test]
    fn empty_slug_is_missing() {
        let git = info(Some("///"), None, None);
        let err = expand("{{branch_slug}}", &git).unwrap_err().to_string();
        assert!(err.contains("branch_slug"), "{err}");
    }

    #[test]
    fn slug_preserves_case_and_safe_chars() {
        assert_eq!(slug("Feat.X_y-1"), "Feat.X_y-1");
    }

    #[test]
    fn apply_branch_override_does_not_touch_commit() {
        let git = info(Some("old"), Some("sha"), Some("url"));
        let out = apply_branch_override(git, Some("feat/x"));
        assert_eq!(out.branch.as_deref(), Some("feat/x"));
        assert_eq!(out.commit.as_deref(), Some("sha"));
        assert_eq!(out.remote.as_deref(), Some("url"));
    }

    #[test]
    fn reject_slash_in_expanded_name() {
        let err = reject_slash_in_name("feature/add-dashboard")
            .unwrap_err()
            .to_string();
        assert!(err.contains("{{branch_slug}}"), "{err}");
    }

    #[test]
    fn multiple_placeholders_in_one_string() {
        let git = info(Some("feat/x"), Some("sha"), Some("git@host:r.git"));
        assert_eq!(
            expand("clone {{remote}} then {{branch}}", &git).unwrap(),
            "clone git@host:r.git then feat/x"
        );
    }

    #[test]
    fn sprite_and_org_placeholders() {
        let git = info(Some("feat"), None, None);
        let vars = SetupVars {
            sprite: Some("myapp-feat"),
            org: Some("acme"),
        };
        assert_eq!(
            expand_in("sprite exec -s {{sprite}} -o {{org}}", &git, vars).unwrap(),
            "sprite exec -s myapp-feat -o acme"
        );
    }

    #[test]
    fn sprite_placeholder_missing() {
        let err = expand_in("{{sprite}}", &GitInfo::default(), SetupVars::default())
            .unwrap_err()
            .to_string();
        assert!(err.contains("sprite"), "{err}");
    }

    #[test]
    fn unclosed_placeholder() {
        let err = expand("{{branch", &info(Some("x"), None, None))
            .unwrap_err()
            .to_string();
        assert!(err.to_lowercase().contains("unclosed"), "{err}");
    }
}
