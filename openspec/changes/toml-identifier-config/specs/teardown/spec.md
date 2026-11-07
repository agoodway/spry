## MODIFIED Requirements

### Requirement: Teardown runs recipe list then destroys

`spry teardown` SHALL load TOML using setup's discovery, `--config`, `--sprite`, `--org`, and `--identifier` inputs. It MUST expand the effective name, require the Sprite CLI, list sprites, then expand each `lifecycle.teardown` command immediately before execution or dry-run printing. Cleanup MUST run in order, fail-fast; destroy MUST follow only after success unless `--no-destroy` is set. VM strings run through Sprite exec; host tables run on the host while the VM still exists. Teardown MUST NOT create a VM or run setup or app start/stop.

Teardown MUST accept `--sprite`, `--org`, `--identifier`, `--dry-run`, `--config`, `--verbose`, and `--no-destroy`. It MUST reject `--branch` and `--no-create`.

Destroy MUST use `sprite destroy -s <name> --force` plus `-o <org>` when set, without a confirmation prompt. A resolved name from `--sprite` or `sprite.name` is required. Org remains optional and MUST be omitted from invocations when absent.

#### Scenario: Teardown commands then destroy

- **WHEN** the VM exists and cleanup has two commands
- **THEN** Spry runs both in order, then destroys with the resolved name and `--force`

#### Scenario: Teardown does not run stop

- **WHEN** setup, teardown, app start, and app stop contain distinct commands and teardown runs
- **THEN** only cleanup commands run before destruction

#### Scenario: Destroy includes org when set

- **WHEN** teardown destroys VM `demo` in org `acme`
- **THEN** destroy includes `-s demo`, `-o acme`, and `--force`

#### Scenario: Org omitted from destroy

- **WHEN** no org is resolved
- **THEN** destroy omits `-o`

#### Scenario: Sprite CLI missing

- **WHEN** teardown runs without the Sprite CLI available
- **THEN** Spry fails before list, exec, host commands, or destroy, with installation guidance

#### Scenario: Name missing

- **WHEN** neither `sprite.name` nor `--sprite` supplies a name
- **THEN** Spry fails with guidance to set `sprite.name` or pass `--sprite`

#### Scenario: Identifier-based cleanup targets the same environment

- **WHEN** `sprite.name` is `app-{{identifier}}` and teardown receives identifier `task-42`
- **THEN** VM and host cleanup resolve identifier to `task-42`, sprite to `app-task-42`, and destroy targets `app-task-42`

### Requirement: Successful teardown reports a summary

After success, teardown MUST print the VM name, destroyed/kept status, command count, and elapsed time. Verbose output MUST include config path, resolved name/org (or absent), identifier (or absent), and complete Sprite/host command lines, including destroy only when it will run. It MUST NOT report Git metadata.

#### Scenario: Success summary destroyed

- **WHEN** teardown runs one command and destroys `demo`
- **THEN** the summary reports `demo`, destroyed status, one command, and elapsed time

#### Scenario: Success summary kept

- **WHEN** teardown succeeds on `demo` with `--no-destroy`
- **THEN** the summary reports `demo` and kept status

#### Scenario: Verbose cleanup without destroy

- **WHEN** teardown runs with `--verbose --no-destroy --identifier task-42`
- **THEN** it reports identifier `task-42`, resolved config/name/org, and cleanup invocation lines, but no destroy action or Git metadata

### Requirement: Refuse destroy when requested

When `--no-destroy` is passed and the VM exists, Spry MUST run `lifecycle.teardown` and MUST NOT destroy. A missing VM MUST fail just as it does without that flag. An empty cleanup list with `--no-destroy` MUST succeed after VM existence is confirmed, without exec or destroy.

#### Scenario: Existing VM with no-destroy

- **WHEN** the user passes `--no-destroy`, the VM exists, and `lifecycle.teardown` has a host command
- **THEN** Spry runs the host command and does not destroy

#### Scenario: Missing VM with no-destroy

- **WHEN** the user passes `--no-destroy` and the VM is absent
- **THEN** Spry fails without exec or destruction

#### Scenario: Empty teardown with no-destroy

- **WHEN** the user passes `--no-destroy`, `lifecycle.teardown` is empty, and the VM exists
- **THEN** Spry succeeds without exec or destruction
