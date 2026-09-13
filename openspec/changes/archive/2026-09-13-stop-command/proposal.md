## Why

`spry setup` can start Sprite services (Phoenix, Postgres) that hold a LiveView WebSocket open and keep the VM billed. There is no recipe-driven way to stop those processes. Operators should drop the same way they start: a repo-local list, same flags, templates, and host vs in-VM steps.

## What Changes

- Add a `stop:` list to the recipe (same item shape as `setup:`: string or `{host: …}`).
- Add `spry stop` with `--sprite`, `--org`, `--dry-run`, `--config`, `--verbose`, `--branch`.
- Do **not** create a missing VM. Fail if the sprite is absent (same hint as `setup --no-create`).
- Empty `stop:` is valid: list, confirm the VM exists, exit success with zero commands.
- Init comments document a `stop:` example. Unknown YAML keys remain ignored.

## Capabilities

### New Capabilities

- `stop`: Command, missing-VM behavior, dry-run, fail-fast execution of the `stop:` list.

### Modified Capabilities

- `config`: Recipe schema includes optional `stop` list.
- `init`: Starter file comments include a `stop:` example.
- `templates`: Placeholders expand in `stop:` lines the same way as `setup:`.

## Impact

- `config`, `cli`, new `stop` module, shared step runner. No new crates. Tests use fake sprite/host clients. Does not destroy the VM or call a platform sleep API (none exists).
