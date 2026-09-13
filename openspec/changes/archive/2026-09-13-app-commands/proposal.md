## Why

`spry stop` is recipe-driven teardown of app processes, but the name sits next to `setup` as if it were a VM lifecycle command. Starting those same processes after a sticky stop currently means re-running `setup`. Operators need a paired start/stop surface under `app`, with the same recipe flexibility as setup.

## What Changes

- **BREAKING**: Replace `spry stop` with `spry app stop`. Same flags, same `stop:` list, same missing-VM (no create) behavior.
- Add `spry app start`, which runs a new recipe `start:` list (same item shape as `setup:` / `stop:`).
- `spry app` is a nested command group; `start` and `stop` share flags (`--sprite`, `--org`, `--dry-run`, `--config`, `--verbose`, `--branch`). No `--no-create`.
- Init comments document `start:` and `stop:` examples. Unknown YAML keys remain ignored.

## Capabilities

### New Capabilities

- `app`: Nested `spry app start` / `spry app stop`, missing-VM behavior, dry-run, fail-fast execution of `start:` / `stop:` lists.

### Modified Capabilities

- `config`: Recipe schema includes optional `start` list (sibling of `stop`).
- `stop`: Command path is `spry app stop` instead of `spry stop`.
- `init`: Starter file comments include a `start:` example alongside `stop:`.

## Impact

- `cli`, `config`, replace `stop` module with shared `app` runner. No new crates. Tests use fake sprite/host clients. Does not destroy the VM or call a platform sleep API.
