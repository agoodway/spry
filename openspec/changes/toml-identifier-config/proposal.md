# Proposal

## Why

An environment should be identified by an explicit caller-supplied label, not by the current Git checkout. Moving recipes to structured TOML separates Sprite identity, environment preparation/cleanup, and app controls while removing implicit Git dependencies.

## What Changes

- **BREAKING**: Replace YAML recipes with `.spry.toml` / `spry.toml`, using `[sprite]` for `name`/`org`, `[lifecycle]` for `setup`/`teardown`, and `[app]` for `start`/`stop`. Reject unknown fields and unsupported legacy recipes with actionable errors.
- Add `--identifier` to setup, app start/stop, and teardown, and `{{identifier}}` to name and command expansion. Require an alphanumeric first character followed by letters, digits, underscores, or hyphens, without rewriting values; require a value only when referenced by the effective name or selected phase.
- Warn on stderr when a selected TOML recipe shadows neighboring legacy YAML, and print a migration note when init writes beside legacy YAML. Keep legacy files unchanged for rollback.
- **BREAKING**: Remove `{{branch}}`, `{{branch_slug}}`, `{{commit}}`, `{{remote}}`, `--branch`, and automatic Git metadata discovery. Keep `{{sprite}}` and `{{org}}` in commands.
- Update `spry init`, help, and documentation to generate and explain TOML recipes without Git-centric examples.
- Preserve explicit, rerunnable setup; independent app start/stop; ordered host/VM steps; dry-run behavior; and cleanup-before-destroy teardown, including `--no-destroy`.
- Do not add a resume command, resume hook, wake listener, or automatic lifecycle execution.

## Capabilities

### New Capabilities

None. Identifier behavior belongs to the existing templates capability.

### Modified Capabilities

- `config`: Structured TOML schema, strict fields, discovery, overrides, and YAML migration diagnostics.
- `host-setup`: TOML host-step syntax with unchanged host execution and environment behavior.
- `templates`: Identifier-based substitution, validation, and removal of Git discovery/placeholders.
- `init`: TOML starter file and identifier-oriented examples.
- `setup`: New identity inputs and TOML lifecycle setup, without changing provisioning semantics.
- `app`: TOML app start/stop lists and identifier flags while preserving independent execution.
- `stop`: Align the existing stop contract with TOML and identifier inputs.
- `teardown`: TOML cleanup list and identifier inputs while retaining teardown safety and destruction behavior.

## Impact

Implementation touches `src/config.rs`, `src/template.rs`, `src/cli.rs`, `src/init.rs`, `src/setup.rs`, `src/app.rs`, `src/teardown.rs`, and shared step handling. Remove the now-unused `src/git.rs` and its module export. Replace `serde_yaml` with a maintained Serde-compatible TOML dependency and update `Cargo.lock`; this intentionally supersedes the existing YAML dependency pin. Update existing tests and README examples. No Sprite API changes or VM operations are needed to plan this change.
