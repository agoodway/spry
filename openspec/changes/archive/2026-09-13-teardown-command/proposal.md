## Why

`spry setup` creates a Sprite VM and can register host-side resources (GitHub deploy keys). `spry app stop` only stops processes. There is no recipe-driven way to destroy the VM or clean up those host resources. Operators need a `setup` counterpart: run optional cleanup, then destroy.

## What Changes

- Add a `teardown:` list to the recipe (same item shape as `setup:`: string or `{host: …}`).
- Add `spry teardown` with `--sprite`, `--org`, `--dry-run`, `--config`, `--verbose`, `--branch`, and `--no-destroy`.
- Do **not** create a missing VM. Fail if the sprite is absent (no teardown steps, no destroy).
- After teardown steps succeed, destroy the VM with `sprite destroy -s <name> --force` (plus `-o <org>` when org is set). No prompt.
- Empty `teardown:` is valid: confirm the VM exists, then destroy.
- `--no-destroy` runs `teardown:` and leaves the VM.
- Fail-fast: a failed teardown step MUST NOT destroy.
- Init comments document a `teardown:` example. Unknown YAML keys remain ignored.

## Capabilities

### New Capabilities

- `teardown`: Command, missing-VM error, dry-run, fail-fast `teardown:` list, then destroy.

### Modified Capabilities

- `config`: Recipe schema includes optional `teardown` list.
- `init`: Starter file comments include a `teardown:` example.
- `templates`: Placeholders expand in `teardown:` lines the same way as `setup:`.

## Impact

- `config`, `cli`, new `teardown` module, `SpriteClient::destroy`. No new crates. Tests use fake sprite/host clients. Does not implicitly run `stop:`. Wraps `sprite destroy`; does not call the Sprites HTTP API.
