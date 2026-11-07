## MODIFIED Requirements

### Requirement: Nested app start and stop commands

`spry app start` and `spry app stop` SHALL load structured TOML using setup's discovery, `--config`, `--sprite`, `--org`, and `--identifier` behavior. They MUST expand the effective name, require the Sprite CLI, list sprites, then expand each selected command immediately before execution or dry-run printing. Only `app.start` or `app.stop` MUST run, in order, fail-fast. VM strings execute via Sprite exec; host tables execute on the host.

Both commands MUST accept `--sprite`, `--org`, `--identifier`, `--dry-run`, `--config`, and `--verbose`, and MUST reject `--branch` and `--no-create`. Top-level `spry stop` MUST remain invalid.

A missing VM MUST fail without creation, step execution, or host commands, with a hint including `sprite create` and `spry setup`. Empty lists MUST succeed after VM existence is confirmed. Dry-run MUST list sprites, print expanded selected steps, avoid executing them, and still fail for a missing VM.

#### Scenario: Missing VM does not create

- **WHEN** app start or stop targets a missing VM
- **THEN** Spry fails without creating or executing and includes `sprite create` and `spry setup` guidance

#### Scenario: Start commands run in order

- **WHEN** `app.start` has two VM commands and the VM exists
- **THEN** app start runs both in order and succeeds

#### Scenario: Stop commands run in order

- **WHEN** `app.stop` has two VM commands and the VM exists
- **THEN** app stop runs both in order and succeeds

#### Scenario: Start does not run stop

- **WHEN** a recipe has distinct setup, start, stop, and teardown commands and the user runs app start
- **THEN** only `app.start` executes

#### Scenario: Empty start on existing VM

- **WHEN** `app.start` is absent or empty and the VM exists
- **THEN** Spry lists sprites and succeeds without exec or host commands

#### Scenario: Empty stop on existing VM

- **WHEN** `app.stop` is absent or empty and the VM exists
- **THEN** Spry lists sprites and succeeds without exec or host commands

#### Scenario: Top-level stop is removed

- **WHEN** the user runs `spry stop`
- **THEN** CLI parsing rejects the command

#### Scenario: Start and stop resolve the same identifier

- **WHEN** `sprite.name` is `app-{{identifier}}` and start and stop each receive identifier `task-42`
- **THEN** both target `app-task-42` and expand identifier/sprite/org in their selected VM and host commands

#### Scenario: App dry-run preserves VM and host ordering

- **WHEN** the selected app list alternates VM and host commands and dry-run is requested on an existing VM
- **THEN** output contains the expanded planned commands in list order and neither executor runs commands

## ADDED Requirements

### Requirement: Explicit lifecycle actions only

Spry SHALL expose setup, app start/stop, and teardown as explicit execution actions. It MUST NOT add a resume command, install wake hooks, or run app controls implicitly from setup or teardown. App controls MUST remain usable independently while the Sprite is active.

#### Scenario: Resume is not a command

- **WHEN** the user invokes `spry resume`
- **THEN** CLI parsing rejects it

#### Scenario: App controls during active use

- **WHEN** the VM already exists and the user invokes app stop followed by app start
- **THEN** only the respective app lists run, with no setup, teardown, or VM destruction
