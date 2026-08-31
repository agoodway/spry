use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::init::{self, InitOpts};
use crate::setup::{self, SetupOpts};
use crate::sprite::{self, CommandClient};

/// Repo-local Sprite VM recipes.
#[derive(Debug, Parser)]
#[command(name = "spry", version, about, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Write a starter `.spry.yaml` in the current directory
    Init(InitArgs),
    /// Provision and configure a sprite VM from the recipe
    Setup(SetupArgs),
}

fn nonempty_cli_value(s: &str) -> Result<String, String> {
    if s.trim().is_empty() {
        Err("must not be empty".into())
    } else {
        Ok(s.to_string())
    }
}

#[derive(Debug, clap::Args)]
pub struct InitArgs {
    /// Overwrite an existing recipe file
    #[arg(long)]
    pub force: bool,
    /// Write to this path instead of `.spry.yaml`
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, clap::Args)]
pub struct SetupArgs {
    /// Sprite VM name (overrides recipe `name`)
    #[arg(short = 's', long = "sprite", value_parser = nonempty_cli_value)]
    pub sprite: Option<String>,
    /// Sprite organization (overrides recipe `org`)
    #[arg(short = 'o', long = "org", value_parser = nonempty_cli_value)]
    pub org: Option<String>,
    /// Do not create the VM if it is missing
    #[arg(long)]
    pub no_create: bool,
    /// Print planned sprite commands without creating or executing
    #[arg(long)]
    pub dry_run: bool,
    /// Load this recipe instead of walking from the current directory
    #[arg(short = 'c', long = "config")]
    pub config: Option<PathBuf>,
    /// Print resolved config and full sprite command lines
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}

/// Parse argv from the process and dispatch.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;
    dispatch(cli, &cwd, &mut io::stdout())
}

pub fn dispatch(cli: Cli, cwd: &Path, out: &mut dyn Write) -> Result<()> {
    match cli.command {
        Commands::Init(args) => {
            let basename = init::dir_basename(cwd)?.to_string();
            init::run(
                &InitOpts {
                    output: args.output,
                    force: args.force,
                },
                cwd,
                &basename,
                out,
            )?;
            Ok(())
        }
        Commands::Setup(args) => {
            let bin = sprite::find_in_path("sprite", std::env::var_os("PATH").as_deref());
            let available = bin.is_some();
            let client = CommandClient::new(bin.unwrap_or_else(|| PathBuf::from("sprite")));
            setup::run(
                &SetupOpts {
                    sprite: args.sprite,
                    org: args.org,
                    no_create: args.no_create,
                    dry_run: args.dry_run,
                    config: args.config,
                    verbose: args.verbose,
                    search_root: None,
                },
                cwd,
                &client,
                available,
                out,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn parses_init_flags() {
        let cli =
            Cli::try_parse_from(["spry", "init", "--force", "--output", "custom.yaml"]).unwrap();
        match cli.command {
            Commands::Init(args) => {
                assert!(args.force);
                assert_eq!(
                    args.output.as_deref(),
                    Some(std::path::Path::new("custom.yaml"))
                );
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn parses_setup_flags() {
        let cli = Cli::try_parse_from([
            "spry",
            "setup",
            "-s",
            "demo",
            "-o",
            "acme",
            "--no-create",
            "--dry-run",
            "-c",
            "recipe.yaml",
            "-v",
        ])
        .unwrap();
        match cli.command {
            Commands::Setup(args) => {
                assert_eq!(args.sprite.as_deref(), Some("demo"));
                assert_eq!(args.org.as_deref(), Some("acme"));
                assert!(args.no_create);
                assert!(args.dry_run);
                assert_eq!(
                    args.config.as_deref(),
                    Some(std::path::Path::new("recipe.yaml"))
                );
                assert!(args.verbose);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn long_form_sprite_and_config() {
        let cli = Cli::try_parse_from([
            "spry",
            "setup",
            "--sprite",
            "n",
            "--org",
            "o",
            "--config",
            "c.yaml",
            "--verbose",
        ])
        .unwrap();
        match cli.command {
            Commands::Setup(args) => {
                assert_eq!(args.sprite.as_deref(), Some("n"));
                assert_eq!(args.org.as_deref(), Some("o"));
                assert_eq!(args.config.as_deref(), Some(std::path::Path::new("c.yaml")));
                assert!(args.verbose);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn rejects_empty_sprite_flag() {
        let err = Cli::try_parse_from(["spry", "setup", "--sprite", ""]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("must not be empty"), "{msg}");
    }

    #[test]
    fn rejects_empty_org_flag() {
        let err = Cli::try_parse_from(["spry", "setup", "--org", ""]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("must not be empty"), "{msg}");
    }

    #[test]
    fn dispatch_init_writes_recipe() {
        let root = TempDir::new().unwrap();
        let cwd = root.path().join("demo");
        fs::create_dir(&cwd).unwrap();
        let cli = Cli::try_parse_from(["spry", "init"]).unwrap();
        let mut out = Vec::new();
        dispatch(cli, &cwd, &mut out).unwrap();
        let printed = String::from_utf8(out).unwrap();
        let path = cwd.join(".spry.yaml");
        assert!(path.is_file());
        assert!(printed.contains(&path.display().to_string()), "{printed}");
        let recipe = crate::config::read_recipe_file(&path).unwrap();
        assert_eq!(recipe.name.as_deref(), Some("demo"));
        assert!(recipe.setup.is_empty());
    }
}
