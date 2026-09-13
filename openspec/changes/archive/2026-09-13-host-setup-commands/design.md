## Context

`gh repo deploy-key add` must run on the laptop. SSH keygen and `git clone` must run on the Sprite. Spry only had in-VM `sprite exec` steps.

## Goals / Non-Goals

**Goals:** Untagged YAML `string` vs `{host: command}`; host via `bash -lc` with `SPRITE`/`ORG`; `{{sprite}}`/`{{org}}` in setup; dry-run prints `host:`.

**Non-Goals:** First-class GitHub API; passing `GH_TOKEN` into the VM; wrapping host `sprite exec` through `SpriteClient`.

## Decisions

1. Untagged enum so existing string lists keep working.
2. Host runner is injected (`CommandHost` / `FakeHostRunner`) like `SpriteClient`.
3. Host `sprite exec` is a nested real CLI call (private key stays on the VM; pubkey is stdout).

## Risks / Trade-offs

- **[Risk] Nested `sprite exec` from a host step is not the fake client.** → Tests record the host script; integration is dry-run plus real `spry setup`.
- **[Trade-off] `bash -lc` required on the host.** → Matches the deploy-key snippet (`set -euo pipefail`, `mktemp`).
