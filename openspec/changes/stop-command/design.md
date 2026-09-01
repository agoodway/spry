## Context

Setup can start `sprite-env` services that hold TCP (LiveView) and keep a Sprite billed. There is no Sprite sleep API. Stopping those services is the operator lever. Stop should reuse setup’s recipe surface.

## Goals / Non-Goals

**Goals:** `stop:` list; `spry stop` with setup’s flags except `--no-create`; fail if VM missing; empty list OK; templates; host vs sprite steps; dry-run/verbose.

**Non-Goals:** Destroy; platform pause; auto-stopping unnamed services; creating a missing VM.

## Decisions

1. Same `SetupStep` untagged items as `setup:`.
2. Shared `steps::run_steps` for exec/host/fail-fast.
3. Missing VM: error with `sprite create` / `spry setup` (never create).
4. `--http-port` services may restart on the next URL hit; `stop` is still sticky until `services start` / `spry setup`.

## Risks / Trade-offs

- **[Risk] Sticky stop vs proxy auto-start.** → Document; recipe authors choose which services to stop.
