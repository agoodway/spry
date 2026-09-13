## Context

`spry stop` runs a recipe `stop:` list against an existing VM (typically `sprite-env services stop app`) so LiveView sockets drop and the Sprite can idle. There is no matching start path: after a sticky stop, operators re-run `setup` or exec `sprite-env services start` by hand. Grouping both under `app` makes the VM (`setup`) vs process (`app start` / `app stop`) split obvious.

## Goals / Non-Goals

**Goals:** Nested `spry app start` / `spry app stop`; recipe `start:` list (same item types as `setup:` / `stop:`); shared flags except `--no-create`; fail if VM missing; empty list OK; templates; host vs sprite steps; dry-run/verbose.

**Non-Goals:** Destroy; platform pause; auto-discovering unnamed services; creating a missing VM; keeping a `spry stop` alias.

## Decisions

1. Nested clap subcommand `app` with `start` and `stop`. Top-level `stop` is removed (**BREAKING**).
2. Top-level YAML `start:` and `stop:` lists (not nested under `app:`), same `SetupStep` untagged items as `setup:`. Matches “flexible like setup.”
3. Shared `app::run(phase, …)` over `steps::run_steps`. Phase selects the list and labels (`start` / `stop`, `Start complete` / `Stop complete`).
4. Missing VM: error with `sprite create` / `spry setup` (never create). Same for start and stop, including `--dry-run`.
5. `--http-port` services may restart on the next URL hit; `app stop` is still sticky until `app start` / `services start` / `spry setup`.

## Risks / Trade-offs

- **[Risk] `spry stop` users.** → Document the rename; no alias (command is new).
- **[Risk] Sticky stop vs proxy auto-start.** → Document; recipe authors choose which services to start/stop.
