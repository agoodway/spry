# spry

A small CLI that stores a repo-local Sprite VM recipe and applies it.

Spry wraps the existing [`sprite`](https://sprite.dev) CLI (`list`, `create --skip-console`, `exec`). It does not call the Sprites HTTP API.

Treat `.spry.yaml` `setup` lines as code you are willing to run inside the VM (Makefile-class trust), except `{host: …}` entries which run on your laptop via `bash -lc`. `sprite list` / `create` / `exec` have no timeout: a hung `sprite` process blocks `spry`.

## Install

```sh
cargo install --path .
```

`sprite` must be on `PATH` for `spry setup` and `spry app`. `spry init` does not need it.

## Quick start

```sh
spry init
spry setup --dry-run
spry setup
spry app stop
spry app start
```

## Recipe

`spry init` writes `.spry.yaml` in the current directory. Discovery walks from the current directory toward the filesystem root (it does not stop at `$HOME`; use `--config` to skip walking), preferring `.spry.yaml` over `spry.yaml` in each directory.

```yaml
name: myapp-{{branch_slug}}
org: example-org
setup:
  - mkdir -p ~/.ssh && ssh-keygen -t ed25519 -N "" -C sprite-example-app -f ~/.ssh/example-app_ed25519
  - host: |
      PUB=$(sprite exec -s "$SPRITE" -o "$ORG" -- cat /home/sprite/.ssh/example-app_ed25519.pub)
      gh repo deploy-key add -R example-org/example-app --title "sprite-$SPRITE" --allow-write /dev/stdin <<< "$PUB"
  - git clone git@github.com:example-org/example-app.git /home/sprite/example-app
  - git -C /home/sprite/example-app checkout "{{branch}}"
start:
  - sprite-env services start app
stop:
  - sprite-env services stop app
```

A plain string runs inside the VM (`sprite exec`). A `{host: …}` map runs on the laptop. Host commands see `SPRITE` (resolved VM name) and `ORG` when org is set.

`name` is required for setup, `app start`, and `app stop` unless you pass `--sprite`. `org` is optional. An omitted or empty `setup` list still provisions the VM. An omitted or empty `start` or `stop` list still requires the VM to exist, then exits success.

### Placeholders

`spry setup`, `spry app start`, and `spry app stop` expand these in the resolved **name** (recipe or `--sprite`) and in each **setup** / **start** / **stop** command, using git from the current directory:

| Token | Value |
|-------|--------|
| `{{branch}}` | Current branch (`git rev-parse --abbrev-ref HEAD`), or `--branch` |
| `{{branch_slug}}` | Same branch with `/` and other unsafe characters turned into `-` (for VM names) |
| `{{commit}}` | Full HEAD SHA |
| `{{remote}}` | `origin` URL |
| `{{sprite}}` | Resolved VM name (setup, start, and stop lines) |
| `{{org}}` | Resolved org (setup, start, and stop lines) |

`--branch` overrides `{{branch}}` / `{{branch_slug}}` even outside a git repo. `{{commit}}` and `{{remote}}` still need a checkout. Values are not shell-quoted — quote them in the recipe. Recipes with no `{{…}}` do not require git.

A sprite name still containing `/` after expansion is an error; use `{{branch_slug}}` (`feature/add-dashboard` → `feature-add-dashboard`).

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

Load the recipe, create the VM if it is missing, then run each setup step (fail-fast). String steps run inside the VM with `sprite exec`. `{host: …}` steps run on the laptop with `bash -lc`. A failure stops later commands; re-running `spry setup` starts again from the first command, so recipes should be idempotent.

```sh
spry setup
spry setup --dry-run
spry setup --no-create
spry setup --sprite demo --org acme
spry setup --config path/to/recipe.yaml
spry setup --verbose
spry setup --branch feat/x
```

- `--dry-run` may list sprites, but does not create, exec, or run host commands. It prints the `sprite` / `host:` lines that would run (after placeholder expansion).
- `--no-create` fails if the VM is missing and prints a `sprite create` command you can run.
- `--verbose` prints the resolved config path, name, org (or that org is absent), template values (or `absent`), and complete `sprite` command lines.
- `--branch` supplies `{{branch}}` / `{{branch_slug}}` instead of the current checkout.

### `spry app start` / `spry app stop`

Load the recipe and run each `start:` or `stop:` step against an **existing** VM (fail-fast). Same item shape as `setup:` (in-VM string or `{host: …}`). Does not create a sprite. A missing VM is an error. `spry app start` runs only `start:`; `spry app stop` runs only `stop:`.

```sh
spry app start
spry app stop
spry app stop --dry-run
spry app start --sprite demo --org acme
spry app stop --config path/to/recipe.yaml
spry app start --verbose
spry app stop --branch feat/x
```

There is no platform sleep API. Stopping services (for example `sprite-env services stop app`) drops LiveView connections so the Sprite can idle. `spry app start` is the matching resume (typically `sprite-env services start app`).

## Flags

| Flag | Command | Meaning |
|------|---------|---------|
| `--sprite`, `-s` | setup, app start, app stop | Override recipe `name` |
| `--org`, `-o` | setup, app start, app stop | Override recipe `org` |
| `--no-create` | setup | Do not provision a missing VM |
| `--dry-run` | setup, app start, app stop | Print planned `sprite` / `host:` invocations |
| `--config`, `-c` | setup, app start, app stop | Load this recipe; skip walk-up discovery |
| `--verbose`, `-v` | setup, app start, app stop | Print resolved inputs and full command lines |
| `--branch` | setup, app start, app stop | Override git branch for `{{branch}}` / `{{branch_slug}}` |
| `--output` | init | Write the starter recipe here |
| `--force` | init | Overwrite an existing recipe file |
