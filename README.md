# spry

A small CLI that stores a repo-local Sprite VM recipe and applies it.

Spry wraps the existing [`sprite`](https://sprite.dev) CLI (`list`, `create --skip-console`, `exec`). It does not call the Sprites HTTP API.

Treat `.spry.yaml` `setup` lines as code you are willing to run inside the VM (Makefile-class trust). `sprite list` / `create` / `exec` have no timeout: a hung `sprite` process blocks `spry`.

## Install

```sh
cargo install --path .
```

`sprite` must be on `PATH` for `spry setup`. `spry init` does not need it.

## Quick start

```sh
spry init
spry setup --dry-run
spry setup
```

## Recipe

`spry init` writes `.spry.yaml` in the current directory. Discovery walks from the current directory toward the filesystem root (it does not stop at `$HOME`; use `--config` to skip walking), preferring `.spry.yaml` over `spry.yaml` in each directory.

```yaml
name: myapp
org: example-org
setup:
  - go install golang.org/x/tools/cmd/stringer@latest
```

`name` is required for setup unless you pass `--sprite`. `org` is optional. An omitted or empty `setup` list still provisions the VM.

## Commands

### `spry init`

Write a starter recipe. The `name` is the current directory basename and `setup` is empty.

```sh
spry init
spry init --output custom.yaml
spry init --force
```

Init does not require `sprite` or a git repository. It refuses to overwrite an existing file unless `--force` is set. `--output` may be an absolute path and can overwrite that path when `--force` is set.

### `spry setup`

Load the recipe, create the VM if it is missing, then run each setup command inside it with `sprite exec` (fail-fast). A failure stops later commands; re-running `spry setup` starts again from the first command, so recipes should be idempotent.

```sh
spry setup
spry setup --dry-run
spry setup --no-create
spry setup --sprite demo --org acme
spry setup --config path/to/recipe.yaml
spry setup --verbose
```

- `--dry-run` may list sprites, but does not create or exec. It prints the `sprite` commands that would run.
- `--no-create` fails if the VM is missing and prints a `sprite create` command you can run.
- `--verbose` prints the resolved config path, name, org (or that org is absent), and complete `sprite` command lines.

## Flags

| Flag | Command | Meaning |
|------|---------|---------|
| `--sprite`, `-s` | setup | Override recipe `name` |
| `--org`, `-o` | setup | Override recipe `org` |
| `--no-create` | setup | Do not provision a missing VM |
| `--dry-run` | setup | Print planned `sprite` invocations |
| `--config`, `-c` | setup | Load this recipe; skip walk-up discovery |
| `--verbose`, `-v` | setup | Print resolved inputs and full command lines |
| `--output` | init | Write the starter recipe here |
| `--force` | init | Overwrite an existing recipe file |
