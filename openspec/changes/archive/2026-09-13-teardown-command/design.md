## Context

`spry setup` creates a Sprite and runs `setup:` (in-VM and host). `spry app start` / `spry app stop` run process lists on an existing VM and never destroy. Host setup can register GitHub deploy keys that outlive the disk. Destroy was an explicit non-goal of the original CLI.

Sprite already has `sprite destroy -s <name> --force`. Spry should wrap that as the inverse of setup, with an optional recipe hook for host cleanup **before** the VM is gone.

## Goals / Non-Goals

**Goals:** `teardown:` list (same item types as `setup:`); `spry teardown` with setup flags except `--no-create`; `--no-destroy` to run steps and keep the VM; fail if VM missing; empty list still destroys; templates; host vs sprite steps; dry-run/verbose; always pass `--force` (no prompt).

**Non-Goals:** Implicit `app stop`; creating a missing VM; idempotent success when the sprite is already gone; interactive confirmation; Sprites HTTP API; `spry destroy` alias.

## Decisions

1. **Top-level `spry teardown`.** VM lifecycle sits next to `setup`, not under `app` (processes).

2. **Steps then destroy.** Run `teardown:` via `steps::run_steps` while the VM exists, then `sprite destroy -s <name> --force` (plus `-o <org>` when org is set). Host cleanup can read VM state or delete a deploy key by title. In-VM last rites are allowed; typical recipes are host-only.

3. **Dedicated `teardown` module**, not an `app` phase. Flow is list → require exists → steps → maybe destroy (mirrors setup’s list → maybe create → steps). `SpriteClient` gains `destroy(name, org)`.

4. **Missing VM is an error** (same family as `app start` / `app stop`): no steps, no destroy, including `--dry-run`. Never create.

5. **Fail-fast before destroy.** A failed teardown step leaves the VM. Destroy-anyway would drop the disk and keep host resources. Re-run after fixing the step; recipes should be idempotent.

6. **`--no-destroy`** is the twin of `--no-create`: require the VM, run `teardown:`, skip destroy. Dry-run with `--no-destroy` prints steps only.

7. **No prompt.** Always `--force` (Sprite CLI has no `-f` shorthand). `--dry-run` is the safety (lists, prints teardown lines and the destroy command, does not exec/host/destroy). Validation failures still fail under dry-run.

8. **Empty `teardown:` still destroys** after confirming the VM exists (inverse of empty `setup:` still creating). `--no-destroy` plus empty list is a no-op success after the existence check.

## Risks / Trade-offs

- **[Risk] Irreversible destroy.** → `--dry-run`; fail-fast so cleanup errors do not destroy; no implicit stop.
- **[Risk] Host cleanup already ran, destroy then fails.** → Re-run; teardown lines must be idempotent.
- **[Risk] Operator already `sprite destroy`’d; deploy key leaks.** → Accepted: missing VM errors instead of running host steps anyway.
- **[Risk] Adding `SpriteCall::Destroy` breaks exhaustive matches in setup tests.** → Update those matches in the same change.
- **[Trade-off] Always `--force`.** → Matches spry’s non-interactive create; document that teardown is destructive.
