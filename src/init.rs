use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::user_error;

#[derive(Debug, Clone, Default)]
pub struct InitOpts {
    pub output: Option<PathBuf>,
    pub force: bool,
}

/// Basename of `dir`, used as the starter recipe `name`.
pub fn dir_basename(dir: &Path) -> Result<&str> {
    let name = dir.file_name().ok_or_else(|| {
        user_error(
            "Could not derive a sprite name from the current directory.",
            "run `spry init` from a named directory (not `/`)",
        )
    })?;
    name.to_str().ok_or_else(|| {
        user_error(
            "Current directory name is not valid UTF-8.",
            "rename the directory to a UTF-8 name, then re-run `spry init`",
        )
    })
}

/// Resolve the init target path against `cwd`.
pub fn target_path(cwd: &Path, output: Option<&Path>) -> PathBuf {
    match output {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => cwd.join(path),
        None => cwd.join(".spry.yaml"),
    }
}

/// Quote a YAML double-quoted scalar so the basename always round-trips as a string.
fn yaml_double_quoted(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Starter recipe YAML. Valid on its own: `name` plus empty `setup`.
pub fn starter_yaml(name: &str) -> String {
    let quoted = yaml_double_quoted(name);
    format!(
        "# Spry recipe — share this file with the repository.\n\
         name: {quoted}\n\
         # name: {name}-{{{{branch_slug}}}}\n\
         # org: your-org\n\
         setup: []\n\
         # setup:\n\
         #   - git clone \"{{{{remote}}}}\" /home/sprite/{name}\n\
         #   - git -C /home/sprite/{name} checkout \"{{{{branch}}}}\"\n\
         # start:\n\
         #   - sprite-env services start app\n\
         # stop:\n\
         #   - sprite-env services stop app\n"
    )
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Could not create directory {}", parent.display()))?;
        }
    }
    Ok(())
}

fn write_new(path: &Path, contents: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|err| {
            if err.kind() == io::ErrorKind::AlreadyExists {
                user_error(
                    format!("Refusing to overwrite {}", path.display()),
                    "re-run with `--force` to replace the existing file",
                )
            } else {
                anyhow::Error::new(err).context(format!("Could not write {}", path.display()))
            }
        })?;
    file.write_all(contents.as_bytes())
        .with_context(|| format!("Could not write {}", path.display()))?;
    Ok(())
}

/// Write a starter recipe. Does not require `sprite` or git.
pub fn run(opts: &InitOpts, cwd: &Path, basename: &str, out: &mut dyn Write) -> Result<PathBuf> {
    let path = target_path(cwd, opts.output.as_deref());
    ensure_parent_dir(&path)?;
    let contents = starter_yaml(basename);
    if opts.force {
        fs::write(&path, &contents)
            .with_context(|| format!("Could not write {}", path.display()))?;
    } else {
        write_new(&path, &contents)?;
    }
    writeln!(out, "Created {}", path.display())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use std::fs;
    use tempfile::TempDir;

    fn init_in(dir: &Path, opts: InitOpts) -> Result<(PathBuf, String)> {
        let basename = dir_basename(dir)?.to_string();
        let mut out = Vec::new();
        let path = run(&opts, dir, &basename, &mut out)?;
        Ok((path, String::from_utf8(out).unwrap()))
    }

    #[test]
    fn default_init() {
        let root = TempDir::new().unwrap();
        let cwd = root.path().join("demo");
        fs::create_dir(&cwd).unwrap();

        let (path, printed) = init_in(&cwd, InitOpts::default()).unwrap();
        assert_eq!(path, cwd.join(".spry.yaml"));
        assert!(printed.contains(&path.display().to_string()), "{printed}");

        let recipe = config::read_recipe_file(&path).unwrap();
        assert_eq!(recipe.name.as_deref(), Some("demo"));
        assert!(recipe.setup.is_empty());
        assert_eq!(recipe.name.as_deref(), Some(dir_basename(&cwd).unwrap()));
        let raw = fs::read_to_string(&path).unwrap();
        assert!(raw.contains("{{branch_slug}}"), "{raw}");
        assert!(raw.contains("{{remote}}"), "{raw}");
        assert!(raw.contains("{{branch}}"), "{raw}");
        assert!(raw.contains("start:"), "{raw}");
        assert!(raw.contains("stop:"), "{raw}");
        assert!(recipe.start.is_empty());
        assert!(recipe.stop.is_empty());
    }

    #[test]
    fn init_does_not_need_sprite_cli() {
        let dir = TempDir::new().unwrap();
        // `run` takes no SpriteClient and never consults PATH.
        let (path, _) = init_in(dir.path(), InitOpts::default()).unwrap();
        assert!(path.is_file());
        config::read_recipe_file(&path).unwrap();
        assert!(!path.parent().unwrap().join("sprite").exists());
    }

    #[test]
    fn init_outside_git() {
        let dir = TempDir::new().unwrap();
        assert!(!dir.path().join(".git").exists());
        let (path, _) = init_in(dir.path(), InitOpts::default()).unwrap();
        assert!(path.is_file());
    }

    #[test]
    fn existing_file_without_force() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join(".spry.yaml");
        fs::write(&target, "name: original\nsetup: []\n").unwrap();

        let err = init_in(dir.path(), InitOpts::default())
            .unwrap_err()
            .to_string();
        assert!(err.contains("--force"), "{err}");
        let contents = fs::read_to_string(&target).unwrap();
        assert_eq!(contents, "name: original\nsetup: []\n");
    }

    #[test]
    fn existing_file_with_force() {
        let root = TempDir::new().unwrap();
        let cwd = root.path().join("demo");
        fs::create_dir(&cwd).unwrap();
        let target = cwd.join(".spry.yaml");
        fs::write(&target, "name: original\nsetup: []\n").unwrap();

        let (path, _) = init_in(
            &cwd,
            InitOpts {
                force: true,
                output: None,
            },
        )
        .unwrap();
        let recipe = config::read_recipe_file(&path).unwrap();
        assert_eq!(recipe.name.as_deref(), Some("demo"));
        assert_ne!(recipe.name.as_deref(), Some("original"));
    }

    #[test]
    fn init_with_output_path() {
        let dir = TempDir::new().unwrap();
        let (path, printed) = init_in(
            dir.path(),
            InitOpts {
                output: Some(PathBuf::from("custom.yaml")),
                force: false,
            },
        )
        .unwrap();
        assert_eq!(path, dir.path().join("custom.yaml"));
        assert!(path.is_file());
        assert!(!dir.path().join(".spry.yaml").exists());
        assert!(printed.contains(&path.display().to_string()), "{printed}");
        let recipe = config::read_recipe_file(&path).unwrap();
        assert_eq!(
            recipe.name.as_deref(),
            Some(dir_basename(dir.path()).unwrap())
        );
        assert!(recipe.setup.is_empty());
    }

    #[test]
    fn output_path_exists_without_force() {
        let dir = TempDir::new().unwrap();
        let custom = dir.path().join("custom.yaml");
        fs::write(&custom, "name: keep-me\nsetup: []\n").unwrap();

        let err = init_in(
            dir.path(),
            InitOpts {
                output: Some(PathBuf::from("custom.yaml")),
                force: false,
            },
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("--force"), "{err}");
        assert_eq!(
            fs::read_to_string(&custom).unwrap(),
            "name: keep-me\nsetup: []\n"
        );
    }

    #[test]
    fn output_path_exists_with_force() {
        let dir = TempDir::new().unwrap();
        let custom = dir.path().join("custom.yaml");
        fs::write(&custom, "name: keep-me\nsetup: []\n").unwrap();

        let (path, _) = init_in(
            dir.path(),
            InitOpts {
                output: Some(PathBuf::from("custom.yaml")),
                force: true,
            },
        )
        .unwrap();
        let recipe = config::read_recipe_file(&path).unwrap();
        assert_eq!(
            recipe.name.as_deref(),
            Some(dir_basename(dir.path()).unwrap())
        );
        assert_ne!(recipe.name.as_deref(), Some("keep-me"));
        assert!(!dir.path().join(".spry.yaml").exists());
    }

    #[test]
    fn yaml_boolean_basename_round_trips() {
        let root = TempDir::new().unwrap();
        let cwd = root.path().join("true");
        fs::create_dir(&cwd).unwrap();
        let (path, _) = init_in(&cwd, InitOpts::default()).unwrap();
        let recipe = config::read_recipe_file(&path).unwrap();
        assert_eq!(recipe.name.as_deref(), Some("true"));
        let raw = fs::read_to_string(&path).unwrap();
        assert!(
            raw.contains("name: \"true\"") || raw.contains("name: 'true'"),
            "{raw}"
        );
    }

    #[test]
    fn yaml_colon_basename_round_trips() {
        let root = TempDir::new().unwrap();
        let cwd = root.path().join("foo: bar");
        fs::create_dir(&cwd).unwrap();
        let (path, _) = init_in(&cwd, InitOpts::default()).unwrap();
        let recipe = config::read_recipe_file(&path).unwrap();
        assert_eq!(recipe.name.as_deref(), Some("foo: bar"));
    }

    #[test]
    fn output_creates_missing_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let (path, _) = init_in(
            dir.path(),
            InitOpts {
                output: Some(PathBuf::from("nested/dir/recipe.yaml")),
                force: false,
            },
        )
        .unwrap();
        assert_eq!(path, dir.path().join("nested/dir/recipe.yaml"));
        assert!(path.is_file());
    }

    #[test]
    fn root_dir_basename_errors() {
        let err = dir_basename(Path::new("/")).unwrap_err().to_string();
        assert!(err.contains("derive a sprite name"), "{err}");
    }
}
