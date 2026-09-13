## 1. Config and runner

- [x] 1.1 Add `start:` to recipe schema (same item types as `setup:` / `stop:`)
- [x] 1.2 Replace `stop` module with shared `app` runner (`start` / `stop` phases)

## 2. CLI

- [x] 2.1 Nested `spry app start` and `spry app stop` with setup flags except `--no-create`; remove top-level `stop`
- [x] 2.2 Missing VM error; empty list; isolation of start vs stop; dry-run; host steps; templates

## 3. Docs and recipe

- [x] 3.1 Init comments, README, example-app recipe `start:` / `stop:`
