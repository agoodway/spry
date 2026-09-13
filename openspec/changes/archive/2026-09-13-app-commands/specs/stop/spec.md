## MODIFIED Requirements

### Requirement: Stop runs recipe stop list on an existing VM

`spry app stop` SHALL load the recipe (same discovery, `--config`, `--sprite`, `--org`, `--branch` as setup), expand placeholders in the name and each stop command, require `sprite` on PATH, list sprites, and run `stop:` steps in order (fail-fast). String steps run via `sprite exec`. `{host: …}` steps run on the host.

The command is `spry app stop`. `spry stop` is not a valid invocation.

If the resolved sprite is missing, Spry MUST fail without creating it. The error MUST include a `sprite create` command and mention `spry setup`.

An omitted or empty `stop` list MUST succeed after confirming the VM exists.

`--dry-run` MUST list sprites, MUST NOT exec or run host commands, and MUST still fail if the VM is missing.

#### Scenario: Missing VM does not create

- **WHEN** the user runs `spry app stop` and `sprite list` does not include the name
- **THEN** Spry fails, does not create, does not exec, and the error includes `sprite create` and `spry setup`

#### Scenario: Stop commands run in order

- **WHEN** the recipe has two stop commands and the VM exists
- **THEN** Spry runs both via exec in order and exits success

#### Scenario: Empty stop on existing VM

- **WHEN** the recipe has a name, no stop commands, and the VM exists
- **THEN** Spry lists sprites, does not exec, and exits success
