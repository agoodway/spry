## Why

Sprite.dev VMs need a repo-local recipe: a named VM, optional org, and a list of shell commands to run inside it. Today that lives as a sidecar in other tools (or as ad-hoc `sprite exec` calls), so setup is easy to skip, hard to share, and does not create the VM when it is missing. Spry should be a small Rust CLI whose only job is that recipe.

## What Changes

- Add a Rust CLI binary `spry` with two commands: `init` and `setup`.
- Add repo-only config (`.spry.yaml` or `spry.yaml`) with `name`, optional `org`, and a `setup` list of shell commands.
- `spry init` writes a starter config in the current directory.
- `spry setup` loads config, ensures the sprite CLI is available, creates the VM if it does not exist, then runs each setup command via `sprite exec` (fail-fast).
- Flags override config (`--sprite`, `--org`). `--no-create` refuses to provision a missing VM. `--dry-run` prints planned `sprite` invocations without executing them.
- No git requirement, no user-level config, no `sprite use`, no workie interop.

## Capabilities

### New Capabilities

- `config`: Repo-only YAML recipe — discovery, schema, and flag/config precedence.
- `init`: Generate a starter `.spry.yaml` in the current directory.
- `setup`: Provision (if needed) and configure a sprite VM from the recipe.

### Modified Capabilities

- None. This is a greenfield CLI.

## Impact

- New Rust crate at the repository root (`spry` binary).
- Depends on the `sprite` CLI being on `PATH` at runtime (wraps `list`, `create`, `exec`; does not call the Sprites API directly).
- No existing application code, APIs, or specs to migrate.
- Tests should stub the `sprite` binary rather than requiring a real VM.
