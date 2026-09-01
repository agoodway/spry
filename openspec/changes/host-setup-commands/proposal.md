## Why

Deploy keys and `gh` must run on the laptop (private key never leaves the Sprite; public key is registered with GitHub). Spry currently wraps every setup line in `sprite exec`, so that split is impossible.

## What Changes

- Setup entries may be a string (run inside the VM) or `{host: <command>}` (run on the host via `bash -lc`).
- Host commands get `SPRITE` and, when set, `ORG` in the environment. Placeholders `{{sprite}}` and `{{org}}` expand to the resolved VM name and org.
- Dry-run / verbose print host lines as `host: …` and do not execute them.
- Fail-fast is unchanged.

## Capabilities

### New Capabilities

- `host-setup`: Host-side setup steps, env, and dry-run.

### Modified Capabilities

- `config`: Setup list items may be strings or `{host: …}` maps.
- `setup`: Dispatch sprite vs host; expand `{{sprite}}` / `{{org}}` in setup lines.
- `templates`: `{{sprite}}` and `{{org}}` in setup (not in recipe `name`).

## Impact

- `config`, `setup`, `template`, new `host` module. No new crates. Tests use a fake host runner (no real `bash` / `gh`).
