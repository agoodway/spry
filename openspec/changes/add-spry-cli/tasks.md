## 1. Crate bootstrap

- [x] 1.1 Initialize a Rust binary crate named `spry` at the repo root (edition 2021+)
- [x] 1.2 Add dependencies: `clap` (derive), `serde`, `serde_yaml`, `anyhow`
- [x] 1.3 Add modules `config`, `sprite`, `init`, `setup`, and `cli` with empty skeletons wired from `main`

## 2. Config loading

- [x] 2.1 Parse recipe YAML into `name`, `org`, and `setup`; ignore unknown keys; treat empty `name`/`org` as absent; treat omitted or empty `setup` as zero commands
- [x] 2.2 Implement walk-up discovery from cwd: `.spry.yaml` then `spry.yaml` per directory; no git; no home config
- [x] 2.3 Load `--config <path>` without walking; resolve relative paths against cwd; error with resolved path if missing
- [x] 2.4 Resolve name/org with `--sprite` / `--org` overriding file values
- [x] 2.5 Tests covering config spec scenarios (valid recipes, unknown keys, empty name, invalid YAML, discovery precedence, missing recipe, explicit path)

## 3. Sprite client

- [x] 3.1 Define `SpriteClient` (`list`, `create`, `exec`) plus a fake implementation that records calls and returns scripted results
- [x] 3.2 Implement PATH check for the `sprite` executable with an error that tells the user to install it
- [x] 3.3 Implement production `list` (`sprite list` + optional `-o`) and parse names as whole trimmed line or first whitespace field
- [x] 3.4 Implement production `create` (`sprite create <name> --skip-console` + optional `-o`) and `exec` (`sprite exec -s <name>` + optional `-o` + `-- sh -c <command>`), streaming stdout/stderr
- [x] 3.5 Unit tests for argv construction (including `--skip-console` on create) and list-name parsing (name-per-line and first-field table)

## 4. Init command

- [x] 4.1 Write a valid starter `.spry.yaml` in cwd with `name` derived from the cwd basename and `setup: []`; print the created path; do not require `sprite` or git
- [x] 4.2 Refuse to overwrite an existing target without `--force`; mention `--force` in the error; overwrite when `--force` is set
- [x] 4.3 Honor `--output <path>` (relative to cwd) with the same overwrite rules
- [x] 4.4 Tests covering init spec scenarios (default write and derived name, no sprite required, no git required, existing file, force, custom output)

## 5. Setup command

- [x] 5.1 Require a resolved sprite name; omit `-o` when org is absent; do not require git
- [x] 5.2 List sprites, create when missing with `--skip-console` (no prompt or interactive console), skip create when present; stop without exec if list or create fails
- [x] 5.3 Honor `--no-create`: error with a `sprite create` hint when missing; proceed when present
- [x] 5.4 Honor `--dry-run`: allow list, never create/exec, print planned commands; still fail validation and `--no-create` + missing VM
- [x] 5.5 Run setup commands in order via `exec`, label every invoked step `[i/n]`, show output, and fail fast with index `i of n`; empty setup exits success after provision
- [x] 5.6 Print a short success summary (name, created vs existed, command count, elapsed time)
- [x] 5.7 Tests covering setup spec scenarios using the fake client (CLI missing, name missing, create/skip including `--skip-console`, create failure, list failure, no-create, dry-run, progress labels, fail-fast, success summary, empty setup, no git)

## 6. CLI wiring

- [x] 6.1 Wire clap subcommands `init` and `setup` with flags `--sprite`/`-s`, `--org`/`-o`, `--no-create`, `--dry-run`, `--force`, `--output`, `--config`/`-c`, `--verbose`/`-v`
- [x] 6.2 Dispatch from `main` through `cli`; `--verbose` prints resolved config path, name/org, and full `sprite` command lines
- [x] 6.3 Format user-facing errors with actionable “To fix this” hints (missing CLI, missing config, missing name, missing VM with `--no-create`)

## 7. Docs

- [x] 7.1 Add a README with install (`cargo install --path .`), `spry init`, `spry setup` (including `--dry-run` / `--no-create`), and a `.spry.yaml` example
