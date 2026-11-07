## MODIFIED Requirements

### Requirement: Stop runs recipe stop list on an existing VM

`spry app stop` SHALL load structured TOML using setup's discovery and `--config`, `--sprite`, `--org`, and `--identifier` inputs. It MUST expand the effective name, require the Sprite CLI, list sprites, then expand each `app.stop` command immediately before execution or dry-run printing. Only stop commands MUST run, in order, fail-fast. Strings execute via Sprite exec and host tables execute on the host. It MUST reject `--branch`; top-level `spry stop` MUST remain invalid.

A missing VM MUST fail without creating or executing, with a hint including `sprite create` and `spry setup`. An omitted or empty stop list MUST succeed after VM existence is confirmed. Dry-run MUST list sprites, avoid exec and host commands, print expanded selected steps, and still fail if the VM is missing.

#### Scenario: Missing VM does not create

- **WHEN** app stop targets a missing VM
- **THEN** Spry fails, does not create or execute, and includes `sprite create` and `spry setup` guidance

#### Scenario: Stop commands run in order

- **WHEN** `app.stop` contains two VM commands and the VM exists
- **THEN** both run in order and Spry succeeds

#### Scenario: Empty stop on existing VM

- **WHEN** `app.stop` is omitted or empty and the VM exists
- **THEN** Spry lists sprites, performs no exec, and succeeds

#### Scenario: Stop does not prepare or destroy the environment

- **WHEN** all phase lists contain distinct commands and app stop runs with identifier `task-42`
- **THEN** only the stop list runs on the resolved target, without setup, app start, teardown, creation, or destruction
