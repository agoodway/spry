## Context

Spry is a greenfield Rust CLI. The repository currently has no application code. Runtime dependency is the existing `sprite` CLI (sprite.dev), which already handles auth, org scoping, VM lifecycle, and remote execution. Spry orchestrates that CLI from a repo-local YAML recipe.

The product is two commands (`init`, `setup`) and one config file. Setup creates the VM when it is missing, then runs a shell-command list inside it.

## Goals / Non-Goals

**Goals:**

- Ship a `spry` binary installable via Cargo.
- Persist a shareable per-repo recipe (`.spry.yaml` / `spry.yaml`).
- `spry init` writes a starter recipe in the current directory.
- `spry setup` loads the recipe, wraps `sprite list` / `create` / `exec`, creates the VM if needed, and runs setup commands fail-fast.
- Actionable errors that tell the user how to fix the failure.
- Tests that do not need a real sprite VM.

**Non-Goals:**

- Talking to the Sprites HTTP API directly.
- User-level or org-wide config (`~/.config/spry`).
- Git repository detection or worktree integration.
- Calling `sprite use` or writing a `.sprite` file.
- Declarative tool catalogs (install `td` as a first-class object). Keep opaque shell strings.
- `pre_setup` / `post_setup` phases.
- Checkpoints, doctor, proxy, destroy, or a default-to-setup root command.
- Interactive confirmation before create.

## Decisions

### 1. Wrap the `sprite` CLI, do not use the API

Spry shells out to `sprite`. Auth, org tokens, and VM mechanics stay in that binary.

- Alternatives considered: Sprites API client (duplicates auth and drifts from the CLI); embedding sprite (wrong ownership).
- Rationale: smallest surface, matches how operators already work.

Invocation contract:

| Operation | Command |
|-----------|---------|
| List | `sprite list` plus `-o <org>` when org is set |
| Create | `sprite create <name> --skip-console` plus `-o <org>` when org is set |
| Exec | `sprite exec -s <name>` plus `-o <org>` when org is set, then `-- sh -c <command>` |

Existence check: a VM exists when any non-empty line of `sprite list` stdout has the target name as the whole trimmed line **or** as the first whitespace-separated field. That covers both a name-per-line listing and a simple table.

### 2. Inject a `SpriteClient` for tests

```text
trait SpriteClient {
    fn list(&self, org: Option<&str>) -> Result<Vec<String>>;
    fn create(&self, name: &str, org: Option<&str>) -> Result<()>;
    fn exec(&self, name: &str, org: Option<&str>, command: &str) -> Result<ExecOutput>;
}
```

Production implementation uses `std::process::Command`. Tests use a fake that records calls and returns scripted results. `sprite` on `PATH` is verified before any client call (`which` / `LookPath` equivalent).

### 3. Repo-only YAML, walk-up discovery

Search from the current working directory toward filesystem root. In each directory, prefer `.spry.yaml` over `spry.yaml`. First file found wins.

`--config <path>` on setup (and as a global option if clap structure is cleaner) bypasses discovery. Relative paths resolve against cwd. Missing explicit path is an error.

No git. No home-directory fallback.

Schema:

```yaml
name: myapp          # required for setup unless --sprite
org: example-org        # optional
setup:                    # optional; empty or omitted means create-only
  - go install golang.org/x/tools/cmd/stringer@latest
```

Unknown keys are ignored (forward compatible). Empty `name` is treated as absent. Empty `org` omits `-o`.

Precedence: `--sprite` / `--org` override file values. `--org` with an empty string is not a thing; omit the flag to keep the file value, or put org in the file.

### 4. One `setup:` list; empty list is valid

There is no middle phase, so there are not two hook lists. `spry setup` with a resolved name and zero commands still lists, creates if missing, and exits success. That is “provision this VM.”

### 5. Silent create, `--no-create` to refuse, `--dry-run` to preview

Missing VM → `sprite create` with no prompt. Agents and humans both get a VM that matches the recipe.

`--no-create`: if the VM is missing, error and print the `sprite create` command they could run. Do not create. Do not exec.

`--dry-run`: may run `sprite list` (read-only). Must not run `create` or `exec`. Prints the commands that would run, in order. Validation errors (no CLI, no config, no name, `--no-create` + missing VM) still fail.

### 6. Fail-fast exec, stream output

Commands run in list order. First non-zero `sprite exec` stops the rest. Stdout/stderr from each `sprite` invocation are shown. Each step is labeled `[i/n]`. After success, print a short summary (VM name, created-or-existed, command count, elapsed).

No continue-on-error. No checksum/idempotency beyond whatever the user’s shell commands already do.

### 7. Rust crate layout and dependencies

Crate name and binary: `spry`. Edition 2021+.

Dependencies:

- `clap` with `derive` (subcommands `init`, `setup`; flags `--sprite`/`-s`, `--org`/`-o`, `--no-create`, `--dry-run`, `--force` on init, `--output` on init, `--config`/`-c`, `--verbose`/`-v`)
- `serde` + `serde_yaml` for the recipe
- `anyhow` for user-facing errors with “To fix this” hints

Suggested modules: `config` (discover + parse), `sprite` (client), `init`, `setup`, `cli` (clap). `main` only parses and dispatches.

`--verbose` prints resolved config path, resolved name/org, and full `sprite …` command lines (default mode already prints progress).

### 8. Init writes cwd only

`spry init` writes `.spry.yaml` in the current directory (or `--output`). It does not walk up, does not merge, and does not require `sprite` on PATH. `--force` overwrites. Without `--force`, an existing target file is an error.

Starter file derives `name` from the current directory basename, includes optional commented `org`, and writes `setup: []` with a commented setup example. This produces valid YAML that is immediately usable in the common case.

## Risks / Trade-offs

- **[Risk] `sprite list` output format is not specified.** → Parse whole line or first field as the name; cover both in tests. If the CLI later prints banners or headers, we may need a tighter parser.
- **[Risk] `sprite create` may not be immediately executable.** → Do not poll. If the first `exec` fails, surface that error. Add wait/retry only if real usage shows a race.
- **[Risk] Fail-fast + non-idempotent recipes re-run earlier commands.** → Accepted. Opaque shell lists cannot skip safely. Document that recipes should be idempotent.
- **[Risk] Wrapping CLI vs API drifts if `sprite` flags change.** → Pin the invocation table in this design; one module owns all `sprite` argv construction.
- **[Trade-off] Silent create can provision an unexpected VM.** → `--dry-run` and `--no-create` exist. Name still comes from config or `--sprite`.
- **[Trade-off] Walk-up discovery can pick a parent recipe from a nested cwd.** → Same pattern as other project CLIs; `--config` overrides.

## Migration Plan

Greenfield. No data to migrate.

1. Add the Cargo project and implement `init` / `setup` against `SpriteClient`.
2. `cargo test` with the fake client.
3. Manual check: `spry init`, then `spry setup --dry-run`, then `spry setup` against a real sprite if available.

Rollback: delete the binary; YAML files are inert without it.

## Open Questions

None that block implementation. `sprite list` formatting will be confirmed against the real CLI during the first integration pass and the parser tightened if needed.
