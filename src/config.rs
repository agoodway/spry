use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::user_error;

/// One setup step: a command inside the sprite, or a command on the host.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum SetupStep {
    Sprite(String),
    Host { host: String },
}

impl SetupStep {
    pub fn command(&self) -> &str {
        match self {
            Self::Sprite(command) => command,
            Self::Host { host } => host,
        }
    }

    pub fn is_host(&self) -> bool {
        matches!(self, Self::Host { .. })
    }
}

/// Parsed recipe after empty-string normalization.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Recipe {
    pub name: Option<String>,
    pub org: Option<String>,
    pub setup: Vec<SetupStep>,
    pub start: Vec<SetupStep>,
    pub stop: Vec<SetupStep>,
}

/// Recipe plus the file it came from, with flag overrides applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig {
    pub path: PathBuf,
    pub name: Option<String>,
    pub org: Option<String>,
    pub setup: Vec<SetupStep>,
    pub start: Vec<SetupStep>,
    pub stop: Vec<SetupStep>,
}

#[derive(Debug, Default, Deserialize)]
struct RecipeFile {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    org: Option<String>,
    #[serde(default)]
    setup: Vec<SetupStep>,
    #[serde(default)]
    start: Vec<SetupStep>,
    #[serde(default)]
    stop: Vec<SetupStep>,
}

/// Parse recipe YAML from `contents`. `source` is used in error messages.
pub fn parse_yaml(contents: &str, source: &Path) -> Result<Recipe> {
    let file: RecipeFile = serde_yaml::from_str(contents)
        .map_err(|err| anyhow!("YAML could not be parsed in {}: {err}", source.display()))?;
    Ok(Recipe {
        name: nonempty(file.name),
        org: nonempty(file.org),
        setup: file.setup,
        start: file.start,
        stop: file.stop,
    })
}

/// Read and parse a recipe file at `path`.
pub fn read_recipe_file(path: &Path) -> Result<Recipe> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("Could not read {}", path.display()))?;
    parse_yaml(&contents, path)
}

/// Walk from `start` toward the filesystem root (or `search_root`) looking for
/// `.spry.yaml` then `spry.yaml` in each directory.
pub fn discover(start: &Path, search_root: Option<&Path>) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let dotted = dir.join(".spry.yaml");
        if dotted.is_file() {
            return Some(dotted);
        }
        let plain = dir.join("spry.yaml");
        if plain.is_file() {
            return Some(plain);
        }
        if let Some(root) = search_root {
            if dir == root {
                return None;
            }
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Resolve a relative config path against `cwd`.
pub fn resolve_config_path(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

/// Load a recipe from `--config` or walk-up discovery, then apply flag overrides.
pub fn load(
    cwd: &Path,
    explicit_config: Option<&Path>,
    sprite_override: Option<&str>,
    org_override: Option<&str>,
    search_root: Option<&Path>,
) -> Result<ResolvedConfig> {
    let path = if let Some(explicit) = explicit_config {
        let resolved = resolve_config_path(cwd, explicit);
        if !resolved.is_file() {
            return Err(user_error(
                format!("Config file not found: {}", resolved.display()),
                "pass `--config` with a path that exists, or run `spry init` to create a recipe",
            ));
        }
        resolved
    } else {
        discover(cwd, search_root).ok_or_else(|| {
            user_error(
                "No `.spry.yaml` or `spry.yaml` found from the current directory to the filesystem root.",
                "run `spry init` to create a recipe, or pass `--config <path>`",
            )
        })?
    };

    let recipe = read_recipe_file(&path)?;
    Ok(apply_overrides(recipe, path, sprite_override, org_override))
}

fn apply_overrides(
    recipe: Recipe,
    path: PathBuf,
    sprite_override: Option<&str>,
    org_override: Option<&str>,
) -> ResolvedConfig {
    ResolvedConfig {
        path,
        name: override_or(sprite_override, recipe.name),
        org: override_or(org_override, recipe.org),
        setup: recipe.setup,
        start: recipe.start,
        stop: recipe.stop,
    }
}

fn override_or(flag: Option<&str>, file: Option<String>) -> Option<String> {
    match flag {
        Some(value) if !value.is_empty() => Some(value.to_string()),
        _ => file,
    }
}

fn nonempty(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, body).unwrap();
        path
    }

    fn load_file(dir: &Path, name: &str) -> Recipe {
        read_recipe_file(&dir.join(name)).unwrap()
    }

    #[test]
    fn valid_recipe_with_all_fields() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            "recipe.yaml",
            "name: demo\norg: acme\nsetup:\n  - echo one\n  - echo two\n",
        );
        let recipe = load_file(dir.path(), "recipe.yaml");
        assert_eq!(recipe.name.as_deref(), Some("demo"));
        assert_eq!(recipe.org.as_deref(), Some("acme"));
        assert_eq!(
            recipe.setup,
            vec![
                SetupStep::Sprite("echo one".to_string()),
                SetupStep::Sprite("echo two".to_string())
            ]
        );
    }

    #[test]
    fn valid_recipe_with_only_name() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), "recipe.yaml", "name: demo\n");
        let recipe = load_file(dir.path(), "recipe.yaml");
        assert_eq!(recipe.name.as_deref(), Some("demo"));
        assert_eq!(recipe.org, None);
        assert!(recipe.setup.is_empty());
        assert!(recipe.start.is_empty());
        assert!(recipe.stop.is_empty());
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), "recipe.yaml", "name: demo\ncolor: blue\n");
        let recipe = load_file(dir.path(), "recipe.yaml");
        assert_eq!(recipe.name.as_deref(), Some("demo"));
    }

    #[test]
    fn empty_name_is_absent() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            "recipe.yaml",
            "name: \"\"\norg: \"\"\nsetup: []\n",
        );
        let recipe = load_file(dir.path(), "recipe.yaml");
        assert_eq!(recipe.name, None);
        assert_eq!(recipe.org, None);
        assert!(recipe.setup.is_empty());
    }

    #[test]
    fn omitted_setup_is_zero_commands() {
        let recipe = parse_yaml("name: demo\n", Path::new("memory.yaml")).unwrap();
        assert!(recipe.setup.is_empty());
    }

    #[test]
    fn host_setup_step() {
        let recipe = parse_yaml(
            "name: demo\nsetup:\n  - echo in-vm\n  - host: gh repo deploy-key add\n",
            Path::new("memory.yaml"),
        )
        .unwrap();
        assert_eq!(
            recipe.setup,
            vec![
                SetupStep::Sprite("echo in-vm".to_string()),
                SetupStep::Host {
                    host: "gh repo deploy-key add".to_string()
                }
            ]
        );
    }

    #[test]
    fn stop_list_parses() {
        let recipe = parse_yaml(
            "name: demo\nstop:\n  - sprite-env services stop app\n  - host: echo done\n",
            Path::new("memory.yaml"),
        )
        .unwrap();
        assert!(recipe.setup.is_empty());
        assert!(recipe.start.is_empty());
        assert_eq!(
            recipe.stop,
            vec![
                SetupStep::Sprite("sprite-env services stop app".to_string()),
                SetupStep::Host {
                    host: "echo done".to_string()
                }
            ]
        );
    }

    #[test]
    fn start_list_parses() {
        let recipe = parse_yaml(
            "name: demo\nstart:\n  - sprite-env services start app\n  - host: echo up\nstop:\n  - echo down\n",
            Path::new("memory.yaml"),
        )
        .unwrap();
        assert!(recipe.setup.is_empty());
        assert_eq!(
            recipe.start,
            vec![
                SetupStep::Sprite("sprite-env services start app".to_string()),
                SetupStep::Host {
                    host: "echo up".to_string()
                }
            ]
        );
        assert_eq!(
            recipe.stop,
            vec![SetupStep::Sprite("echo down".to_string())]
        );
    }

    #[test]
    fn invalid_yaml() {
        let dir = TempDir::new().unwrap();
        let path = write(dir.path(), "bad.yaml", "name: [unterminated\n");
        let err = read_recipe_file(&path).unwrap_err().to_string();
        assert!(err.contains(&path.display().to_string()), "{err}");
        assert!(
            err.to_lowercase().contains("yaml could not be parsed"),
            "{err}"
        );
    }

    #[test]
    fn config_in_current_directory() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".spry.yaml", "name: here\n");
        let loaded = load(dir.path(), None, None, None, Some(dir.path())).unwrap();
        assert_eq!(loaded.path, dir.path().join(".spry.yaml"));
        assert_eq!(loaded.name.as_deref(), Some("here"));
    }

    #[test]
    fn prefers_dotted_name_in_the_same_directory() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".spry.yaml", "name: dotted\n");
        write(dir.path(), "spry.yaml", "name: plain\n");
        let loaded = load(dir.path(), None, None, None, Some(dir.path())).unwrap();
        assert_eq!(loaded.name.as_deref(), Some("dotted"));
        assert_eq!(loaded.path.file_name().unwrap(), ".spry.yaml");
    }

    #[test]
    fn walks_up_to_a_parent() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".spry.yaml", "name: parent\n");
        let child = dir.path().join("nested");
        fs::create_dir(&child).unwrap();
        let loaded = load(&child, None, None, None, Some(dir.path())).unwrap();
        assert_eq!(loaded.name.as_deref(), Some("parent"));
        assert_eq!(loaded.path, dir.path().join(".spry.yaml"));
    }

    #[test]
    fn child_wins_over_parent() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".spry.yaml", "name: parent\n");
        let child = dir.path().join("nested");
        fs::create_dir(&child).unwrap();
        write(&child, "spry.yaml", "name: child\n");
        let loaded = load(&child, None, None, None, Some(dir.path())).unwrap();
        assert_eq!(loaded.name.as_deref(), Some("child"));
        assert_eq!(loaded.path, child.join("spry.yaml"));
    }

    #[test]
    fn no_recipe_found() {
        let dir = TempDir::new().unwrap();
        let err = load(dir.path(), None, None, None, Some(dir.path()))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("spry init") || err.contains("--config"),
            "{err}"
        );
        assert!(err.contains("To fix this"), "{err}");
    }

    #[test]
    fn custom_config_path() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".spry.yaml", "name: cwd\n");
        let custom = write(dir.path(), "recipe.yaml", "name: custom\n");
        let loaded = load(
            dir.path(),
            Some(Path::new("recipe.yaml")),
            None,
            None,
            Some(dir.path()),
        )
        .unwrap();
        assert_eq!(loaded.name.as_deref(), Some("custom"));
        assert_eq!(loaded.path, custom);
    }

    #[test]
    fn missing_custom_config_path() {
        let dir = TempDir::new().unwrap();
        let err = load(
            dir.path(),
            Some(Path::new("missing.yaml")),
            None,
            None,
            Some(dir.path()),
        )
        .unwrap_err()
        .to_string();
        let resolved = dir.path().join("missing.yaml");
        assert!(err.contains(&resolved.display().to_string()), "{err}");
    }

    #[test]
    fn flag_overrides_name() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".spry.yaml", "name: from-file\norg: file-org\n");
        let loaded = load(dir.path(), None, Some("from-flag"), None, Some(dir.path())).unwrap();
        assert_eq!(loaded.name.as_deref(), Some("from-flag"));
        assert_eq!(loaded.org.as_deref(), Some("file-org"));
    }

    #[test]
    fn flag_overrides_org() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".spry.yaml", "name: from-file\norg: file-org\n");
        let loaded = load(dir.path(), None, None, Some("flag-org"), Some(dir.path())).unwrap();
        assert_eq!(loaded.org.as_deref(), Some("flag-org"));
        assert_eq!(loaded.name.as_deref(), Some("from-file"));
    }

    #[test]
    fn file_values_used_when_flags_omitted() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".spry.yaml", "name: from-file\norg: file-org\n");
        let loaded = load(dir.path(), None, None, None, Some(dir.path())).unwrap();
        assert_eq!(loaded.name.as_deref(), Some("from-file"));
        assert_eq!(loaded.org.as_deref(), Some("file-org"));
    }

    #[test]
    fn empty_flag_does_not_wipe_file_value() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".spry.yaml", "name: from-file\norg: file-org\n");
        let loaded = load(dir.path(), None, Some(""), Some(""), Some(dir.path())).unwrap();
        assert_eq!(loaded.name.as_deref(), Some("from-file"));
        assert_eq!(loaded.org.as_deref(), Some("file-org"));
    }

    #[test]
    fn absolute_config_path() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".spry.yaml", "name: cwd\n");
        let custom = write(dir.path(), "recipe.yaml", "name: abs\n");
        let loaded = load(dir.path(), Some(&custom), None, None, Some(dir.path())).unwrap();
        assert_eq!(loaded.name.as_deref(), Some("abs"));
        assert_eq!(loaded.path, custom);
        assert!(custom.is_absolute());
    }
}
