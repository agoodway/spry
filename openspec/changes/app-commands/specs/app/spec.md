## ADDED Requirements

### Requirement: Nested app start and stop commands

`spry app start` and `spry app stop` SHALL load the recipe (same discovery, `--config`, `--sprite`, `--org`, `--branch` as setup), expand placeholders in the name and each command, require `sprite` on PATH, list sprites, and run the matching recipe list in order (fail-fast). `start` runs `start:`. `stop` runs `stop:`. String steps run via `sprite exec`. `{host: …}` steps run on the host.

The commands MUST accept `--sprite`, `--org`, `--dry-run`, `--config`, `--verbose`, and `--branch`. They MUST NOT accept `--no-create`.

`spry stop` MUST NOT remain a top-level command.

If the resolved sprite is missing, Spry MUST fail without creating it. The error MUST include a `sprite create` command and mention `spry setup`.

An omitted or empty matching list MUST succeed after confirming the VM exists. The other list MUST NOT run.

`--dry-run` MUST list sprites, MUST NOT exec or run host commands, and MUST still fail if the VM is missing.

#### Scenario: Missing VM does not create

- **WHEN** the user runs `spry app start` or `spry app stop` and `sprite list` does not include the name
- **THEN** Spry fails, does not create, does not exec, and the error includes `sprite create` and `spry setup`

#### Scenario: Start commands run in order

- **WHEN** the recipe has two start commands and the VM exists
- **THEN** `spry app start` runs both via exec in order and exits success

#### Scenario: Stop commands run in order

- **WHEN** the recipe has two stop commands and the VM exists
- **THEN** `spry app stop` runs both via exec in order and exits success

#### Scenario: Start does not run stop

- **WHEN** the recipe has both `start:` and `stop:` commands and the user runs `spry app start`
- **THEN** Spry runs only the start commands

#### Scenario: Empty start on existing VM

- **WHEN** the recipe has a name, no start commands, and the VM exists
- **THEN** `spry app start` lists sprites, does not exec, and exits success

#### Scenario: Empty stop on existing VM

- **WHEN** the recipe has a name, no stop commands, and the VM exists
- **THEN** `spry app stop` lists sprites, does not exec, and exits success

#### Scenario: Top-level stop is removed

- **WHEN** the user runs `spry stop`
- **THEN** Spry fails to parse the command (it is not a valid top-level subcommand)
