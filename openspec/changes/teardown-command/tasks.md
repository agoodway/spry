## 1. Config and sprite client

- [x] 1.1 Add `teardown:` to recipe schema (same item types as `setup:` / `start:` / `stop:`)
- [x] 1.2 Add `SpriteClient::destroy`, `destroy_argv` (`destroy -s <name> --force` plus `-o` when org is set), and `SpriteCall::Destroy`; update exhaustive matches in existing tests

## 2. Teardown command

- [x] 2.1 `teardown` module: load recipe, require name and sprite CLI, list, error if VM missing, run `teardown:` via `steps::run_steps`, then destroy unless `--no-destroy`
- [x] 2.2 Missing VM error (including dry-run and `--no-destroy`); empty list still destroys; fail-fast does not destroy; destroy failure after steps
- [x] 2.3 Dry-run prints expanded teardown lines then destroy (omit destroy with `--no-destroy`); verbose; success summary (destroyed vs kept); templates including a missing placeholder that must not destroy

## 3. CLI and docs

- [x] 3.1 Top-level `spry teardown` with setup flags except `--no-create`; add `--no-destroy`
- [x] 3.2 Init comments for `teardown:`; README (command, flags, example host cleanup)
