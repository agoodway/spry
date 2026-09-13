## Context

Spry setup runs opaque shell commands via `sprite exec -- sh -c <command>` and takes the VM name from the recipe or `--sprite`. Recipes are repo-local and shared across branches. Users were hardcoding the current git branch into `name` and `setup`, which does not travel, and a raw branch like `feature/add-dashboard` is not a valid sprite name (`/` is rejected).

Git remains optional: `spry init`, empty setup, and recipes with no placeholders must keep working without a repository.

## Goals / Non-Goals

**Goals:**

- Expand a small set of placeholders in the resolved sprite **name** and each **setup** command.
- Give each branch a unique, identifiable VM via `{{branch_slug}}` in `name`.
- Discover git from `spry setup`’s cwd; `--branch` overrides branch (and therefore slug).
- Fail the value that needs a missing placeholder, with a “To fix this” hint.
- Dry-run and verbose show expanded values. Tests still use the fake sprite client.

**Non-Goals:**

- Built-in clone/checkout (users still write git lines).
- Env-var injection (`SPRY_BRANCH`) as the public interface.
- Templates in `org` (v1).
- Recursive expansion, escaping a literal `{{branch}}`, or `{{ branch }}` with spaces.
- Calling `sprite use`, writing `.sprite`, or talking to the Sprites HTTP API.
- Auto-slugging `{{branch}}` itself; slug is a distinct placeholder.

## Decisions

### 1. YAML placeholders, not env vars

Public interface is `{{name}}` inside recipe (and `--sprite` / `--branch`) strings. Alternatives: `sprite exec --env` (comma-separated values break; less visible in the recipe) and a first-class clone step (too much git in spry). Expansion is a single raw replace; authors quote in the recipe (`"{{branch}}"`).

Placeholders (case-sensitive, no interior whitespace):

| Token | Source |
|-------|--------|
| `{{branch}}` | `git rev-parse --abbrev-ref HEAD` from cwd, unless `--branch` is set |
| `{{branch_slug}}` | slug of the same branch value |
| `{{commit}}` | `git rev-parse HEAD` (full SHA) |
| `{{remote}}` | `git remote get-url origin` |

Unknown `{{foo}}` is an error. Detached HEAD (`abbrev-ref` is `HEAD`) treats `{{branch}}` / `{{branch_slug}}` as missing unless `--branch` is set; `{{commit}}` still works.

### 2. Slug rules for VM names

Sprite names must not contain `/`. `{{branch_slug}}` is:

1. Take the resolved branch string (after `--branch`).
2. Replace each run of characters outside `[A-Za-z0-9._-]` with a single `-`.
3. Strip leading and trailing `-`.

`feature/add-dashboard` → `feature-add-dashboard`. Case is preserved.

If the expanded **name** still contains `/` (for example `name: "{{branch}}"`), fail and tell the user to use `{{branch_slug}}`. Empty slug after stripping is a missing-value error.

`--sprite` overrides recipe `name` and is expanded the same way, so `--sprite 'app-{{branch_slug}}'` works. A literal `--sprite demo` has no placeholders and does not need git.

### 3. When expansion happens

```text
load recipe + flag overrides
  → expand name (fail here if name uses a missing placeholder)
  → require a non-empty name
  → sprite list / maybe create
  → for each setup line: expand, then exec or dry-run print
```

Provision does not require git unless `name` (or `--sprite`) contains placeholders. A later setup line that cannot expand fails that command; earlier execs may already have run (same fail-fast as a bad `sprite exec`).

Git is read from **cwd**, not from the recipe file path (`--config /tmp/recipe.yaml` still uses the repo you are standing in). `--branch` supplies `{{branch}}` / `{{branch_slug}}` even without git; `{{commit}}` and `{{remote}}` still need a repo.

### 4. Inject git/template context for tests

```text
struct GitInfo { branch, commit, remote }  // each Option<String>
fn git_info(cwd) -> GitInfo                 // empty if git missing or fails
fn slug(branch) -> String
fn expand(template, ctx) -> Result<String>  // ctx = GitInfo + branch override
```

Production setup calls `git_info(cwd)`. Tests pass a `GitInfo` seam on `SetupOpts` so they do not need `git` on PATH except in `git_info` unit tests (temp repo).

`git` is invoked as `git -C <cwd> …` with no extra flags. Non-zero exit or missing binary → that field is absent, not a hard error until a placeholder needs it.

### 5. Verbose and init comments

`--verbose` prints `branch`, `branch_slug`, `commit`, and `remote` as the resolved value or `absent`. Dry-run prints expanded `sprite exec` lines.

`spry init` still writes `name: <cwd basename>` and `setup: []`. Comments MAY show:

```yaml
# name: myapp-{{branch_slug}}
# setup:
#   - git clone "{{remote}}" /home/sprite/myapp
#   - git -C /home/sprite/myapp checkout "{{branch}}"
```

Init does not expand templates and does not run git.

## Risks / Trade-offs

- **[Risk] `git` output formats / encodings.** → Trim stdout; treat empty as absent; tests cover origin URL and slashed branch.
- **[Risk] Branch names that slug to a collision** (`feat/x` and `feat-x`). → Accepted; document. `--sprite` still overrides.
- **[Risk] Raw substitution into `sh -c`.** → Authors must quote; same as today’s opaque commands.
- **[Trade-off] No org templates.** → YAGNI; add later if needed.
- **[Trade-off] Expansion of `--sprite`.** → Consistent with name-from-file; literal names stay literal.

## Migration Plan

No stored data to migrate. Existing recipes without `{{` are unchanged.

1. Add `template` / `git` modules and `--branch`.
2. Expand name before list; expand each setup line before exec.
3. `cargo test`; README + init comments.

Rollback: revert the binary; YAML with `{{` would be executed literally by the old binary.

## Open Questions

None. Sprite name character set is enforced only for `/` after expansion plus slug alphabet above; if `sprite create` rejects more characters, tighten the slug later.
