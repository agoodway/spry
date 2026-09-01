## ADDED Requirements

### Requirement: Teardown runs recipe list then destroys

`spry teardown` SHALL load the recipe (same discovery, `--config`, `--sprite`, `--org`, `--branch` as setup), expand placeholders in the name and each teardown command, require `sprite` on PATH, list sprites, run `teardown:` steps in order (fail-fast), then destroy the VM.

String steps run via `sprite exec`. `{host: …}` steps run on the host. Teardown MUST NOT run `start:` or `stop:`. Teardown MUST NOT create a sprite.

The command MUST accept `--sprite`, `--org`, `--dry-run`, `--config`, `--verbose`, `--branch`, and `--no-destroy`. It MUST NOT accept `--no-create`.

Destroy MUST be `sprite destroy -s <name> --force` plus `-o <org>` when org is set. Spry MUST NOT prompt for confirmation.

A sprite name is required (`--sprite` or recipe `name`). Org is optional; when absent, Spry MUST omit `-o` from sprite invocations.

#### Scenario: Teardown commands then destroy

- **WHEN** the recipe has two teardown commands and the VM exists
- **THEN** Spry runs both in order, then runs `sprite destroy -s` with the name and `--force`, and exits success

#### Scenario: Teardown does not run stop

- **WHEN** the recipe has both `teardown:` and `stop:` commands and the user runs `spry teardown`
- **THEN** Spry runs only the teardown commands, then destroy

#### Scenario: Destroy includes org when set

- **WHEN** the resolved org is `acme` and teardown destroys VM `demo`
- **THEN** the destroy invocation includes `-s demo`, `-o acme`, and `--force`

#### Scenario: Org omitted from destroy

- **WHEN** the resolved org is absent and teardown destroys
- **THEN** Spry does not pass `-o` to `sprite destroy`

#### Scenario: Sprite CLI missing

- **WHEN** the user runs `spry teardown` and `sprite` is not on PATH
- **THEN** Spry fails without listing, exec, host commands, or destroy, and the error mentions installing the sprite CLI

#### Scenario: Name missing

- **WHEN** the recipe has no name and the user does not pass `--sprite`
- **THEN** Spry fails and the error mentions setting `name` or passing `--sprite`

### Requirement: Missing VM is an error

If the resolved sprite is missing, Spry MUST fail without creating, executing teardown steps, running host commands, or destroying. The error MUST say the sprite does not exist.

`--dry-run` MUST still fail if the VM is missing.

#### Scenario: Missing VM does not destroy

- **WHEN** the user runs `spry teardown` and `sprite list` does not include the name
- **THEN** Spry fails, does not exec, does not run host commands, does not destroy, and the error says the sprite does not exist

#### Scenario: Dry-run missing VM still fails

- **WHEN** the user passes `--dry-run` and the VM does not exist
- **THEN** Spry fails without printing destroy as an action it will take and without destroying

### Requirement: Failed teardown step does not destroy

If a teardown step exits non-zero, Spry MUST stop, MUST NOT run later teardown steps, and MUST NOT destroy. The error MUST identify the failing command’s index and the phase as teardown.

#### Scenario: Middle command fails

- **WHEN** the recipe has three teardown commands, the VM exists, and the second step exits non-zero
- **THEN** Spry does not run the third command, does not destroy, and the error identifies command 2 of 3 as teardown

#### Scenario: Destroy failure after steps

- **WHEN** teardown steps succeed and `sprite destroy` exits non-zero
- **THEN** Spry fails and does not retry destroy

### Requirement: Empty teardown still destroys

An omitted or empty `teardown` list MUST succeed after confirming the VM exists, then destroy, without `sprite exec`.

#### Scenario: Empty teardown on existing VM

- **WHEN** the recipe has a name, no teardown commands, and the VM exists
- **THEN** Spry lists sprites, does not exec, runs destroy, and exits success

### Requirement: Refuse destroy when requested

When `--no-destroy` is passed and the VM exists, Spry MUST run `teardown:` steps and MUST NOT destroy.

When `--no-destroy` is passed and the VM is missing, Spry MUST fail the same way as a missing VM without `--no-destroy`.

An empty `teardown` list with `--no-destroy` MUST succeed after confirming the VM exists, without exec or destroy.

#### Scenario: Existing VM with no-destroy

- **WHEN** the user passes `--no-destroy`, the VM exists, and teardown has a host command
- **THEN** Spry runs the host command and does not destroy

#### Scenario: Missing VM with no-destroy

- **WHEN** the user passes `--no-destroy` and `sprite list` does not include the name
- **THEN** Spry fails, does not exec, and does not destroy

#### Scenario: Empty teardown with no-destroy

- **WHEN** the user passes `--no-destroy`, the recipe has no teardown commands, and the VM exists
- **THEN** Spry exits success without exec or destroy

### Requirement: Dry-run previews without mutating

`--dry-run` MAY run `sprite list`. It MUST NOT run `sprite exec`, host commands, or `sprite destroy`. It MUST print the teardown lines that would have run (expanded), then the destroy command, in that order.

When `--dry-run` is combined with `--no-destroy`, Spry MUST print teardown lines and MUST NOT print destroy as an action it will take.

Validation failures (missing sprite CLI, missing config, missing name, missing VM) MUST still fail under `--dry-run`.

#### Scenario: Dry-run with existing VM

- **WHEN** the user passes `--dry-run`, the VM exists, and teardown has an in-VM command and a host command
- **THEN** Spry prints the `sprite exec` line, a `host:` line, and a `sprite destroy` line with `--force`, and does not exec, run host commands, or destroy

#### Scenario: Dry-run with no-destroy

- **WHEN** the user passes `--dry-run --no-destroy`, the VM exists, and teardown has one command
- **THEN** Spry prints that command and does not print `sprite destroy` as an action it will take

### Requirement: Successful teardown reports a summary

After teardown succeeds, Spry MUST print a short summary containing the VM name, whether the VM was destroyed or kept, the number of teardown commands run, and elapsed time.

`--verbose` MUST print the resolved config path, resolved sprite name and org (or that org is absent), template values, and complete `sprite` / `host:` command lines including destroy when destroy will run.

#### Scenario: Success summary destroyed

- **WHEN** teardown succeeds after running one command and destroying VM `demo`
- **THEN** the summary names `demo`, reports that it was destroyed, reports one command, and includes elapsed time

#### Scenario: Success summary kept

- **WHEN** teardown succeeds with `--no-destroy` on VM `demo`
- **THEN** the summary names `demo` and reports that it was kept
