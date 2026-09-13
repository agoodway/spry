## Why

A repo-local recipe is meant to be shared across branches, but setup today is opaque strings. Checking out “this” branch or giving the VM a unique name requires hardcoding `feature/add-dashboard` (and `/` is not a valid sprite name). Spry should fill those values from the local git checkout at `spry setup` time.

## What Changes

- Expand `{{branch}}`, `{{branch_slug}}`, `{{commit}}`, and `{{remote}}` in the resolved sprite **name** and in each **setup** command.
- Discover git from the process cwd of `spry setup`. `--branch` overrides `{{branch}}` / `{{branch_slug}}`. `--sprite` is still expanded if it contains placeholders.
- `{{branch_slug}}` replaces `/` (and other non-name-safe characters) so each branch can own a distinct VM (`myapp-{{branch_slug}}`).
- Fail the value that needs a missing placeholder (name before provision; a setup line at exec / dry-run print time). Recipes with no placeholders still do not require git.
- `--verbose` prints resolved template values. Dry-run prints expanded command lines.
- Document placeholders in the README and in commented `spry init` examples.

## Capabilities

### New Capabilities

- `templates`: Placeholder discovery, expansion, slug rules, `--branch`, and missing-value errors.

### Modified Capabilities

- `setup`: Resolve templated `name` before list/create; expand each setup command before exec/dry-run print; verbose includes template context.
- `init`: Starter recipe comments show `{{branch_slug}}` in `name` and git examples using `{{remote}}` / `{{branch}}`. Init still does not require git.

## Impact

- New modules for git context and template expansion; `--branch` on `spry setup`.
- `sprite` invocations use expanded strings. No new crates. Tests stub git info or use a temp git repo; they still must not need a real sprite VM.
