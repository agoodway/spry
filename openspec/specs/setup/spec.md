## Purpose

Define `spry setup`: ensuring a sprite VM exists and running recipe setup commands fail-fast.

## Requirements

### Requirement: Sprite CLI must be available

`spry setup` SHALL verify that the `sprite` executable is on PATH before listing, creating, or executing. If it is not found, Spry MUST fail with an error that tells the user to install the sprite CLI.

#### Scenario: Sprite CLI missing

- **WHEN** the user runs `spry setup` and `sprite` is not on PATH
- **THEN** Spry fails without attempting list, create, or exec, and the error mentions installing the sprite CLI

#### Scenario: Sprite CLI present

- **WHEN** `sprite` is on PATH and a valid recipe resolves a name
- **THEN** Spry proceeds to list sprites

### Requirement: Sprite name is required

`spry setup` MUST resolve a sprite name from `--sprite` or the recipe `name`. If neither is present, Spry MUST fail with an error that tells the user to set `name` in the recipe or pass `--sprite`.

Org is optional. When org is absent, Spry MUST omit `-o` from sprite invocations.

#### Scenario: Name missing

- **WHEN** the recipe has no name and the user does not pass `--sprite`
- **THEN** Spry fails and the error mentions setting `name` or passing `--sprite`

#### Scenario: Org omitted from invocations

- **WHEN** the resolved org is absent
- **THEN** Spry does not pass `-o` to `sprite list`, `sprite create`, or `sprite exec`

### Requirement: Create the VM when it does not exist

After a successful `sprite list`, if the resolved name is not present, `spry setup` MUST run `sprite create <name> --skip-console` (with `-o <org>` when org is set) and then continue to run setup commands. Spry MUST NOT prompt for confirmation or enter an interactive console.

A VM is present when any non-empty line of `sprite list` stdout equals the name, or the first whitespace-separated field of a line equals the name.

#### Scenario: Missing VM is created

- **WHEN** `sprite list` does not include the resolved name
- **THEN** Spry runs `sprite create` for that name with `--skip-console` (and org if set) and then runs setup commands

#### Scenario: Existing VM is not created

- **WHEN** `sprite list` includes the resolved name
- **THEN** Spry does not run `sprite create` and does run setup commands

#### Scenario: Create failure stops setup

- **WHEN** `sprite create` exits non-zero
- **THEN** Spry fails and does not run setup commands

#### Scenario: List failure stops setup

- **WHEN** `sprite list` exits non-zero
- **THEN** Spry fails and does not create or exec

### Requirement: Refuse create when requested

When `--no-create` is passed and the VM is missing, `spry setup` MUST fail without creating or executing. The error MUST include a `sprite create` command the user can run.

When `--no-create` is passed and the VM exists, setup MUST proceed normally.

#### Scenario: Missing VM with no-create

- **WHEN** the user passes `--no-create` and `sprite list` does not include the name
- **THEN** Spry fails, does not create, does not exec, and the error includes `sprite create`

#### Scenario: Existing VM with no-create

- **WHEN** the user passes `--no-create` and `sprite list` includes the name
- **THEN** Spry runs the setup commands and does not create

### Requirement: Dry-run previews without mutating

`--dry-run` MAY run `sprite list`. It MUST NOT run `sprite create` or `sprite exec`. It MUST print the create and/or exec commands that would have run, in order.

Validation failures (missing sprite CLI, missing config, missing name, `--no-create` with a missing VM) MUST still fail under `--dry-run`.

#### Scenario: Dry-run with missing VM

- **WHEN** the user passes `--dry-run` and the VM does not exist
- **THEN** Spry prints a `sprite create` command and each `sprite exec` command, does not create, and does not exec

#### Scenario: Dry-run with existing VM

- **WHEN** the user passes `--dry-run` and the VM exists
- **THEN** Spry prints each `sprite exec` command, does not print create as an action it will take, and does not exec

#### Scenario: Dry-run still enforces no-create

- **WHEN** the user passes `--dry-run --no-create` and the VM does not exist
- **THEN** Spry fails without creating or executing

### Requirement: Run setup commands in order and fail fast

Each entry in `setup` MUST be executed as:

`sprite exec -s <name> [-o <org>] -- sh -c <command>`

Commands MUST run in list order. Each invoked setup command MUST be labeled `[i/n]`. Stdout and stderr from each invocation MUST be shown. If a command exits non-zero, Spry MUST stop and MUST NOT run later commands. The error MUST identify the failing command’s index and the phase as setup.

#### Scenario: All commands succeed

- **WHEN** the recipe has three setup commands and each `sprite exec` exits zero
- **THEN** Spry labels the commands `[1/3]`, `[2/3]`, and `[3/3]`, runs all three in order, and exits success

#### Scenario: Middle command fails

- **WHEN** the recipe has three setup commands and the second `sprite exec` exits non-zero
- **THEN** Spry does not run the third command and the error identifies command 2 of 3

#### Scenario: Exec includes org when set

- **WHEN** the resolved org is `acme` and a setup command runs
- **THEN** the `sprite exec` invocation includes `-s` with the name and `-o acme`

### Requirement: Successful setup reports a summary

After setup succeeds, Spry MUST print a short summary containing the VM name, whether the VM was created or already existed, the number of setup commands run, and elapsed time.

#### Scenario: Success summary

- **WHEN** setup succeeds after creating VM `demo` and running two setup commands
- **THEN** the summary names `demo`, reports that it was created, reports two commands, and includes elapsed time

### Requirement: Verbose setup reports resolved inputs and invocations

When `--verbose` is passed to `spry setup`, Spry MUST print the resolved config path, resolved sprite name and org, and the complete `sprite` command line for every invocation. When org is absent, verbose output MUST identify it as absent rather than inventing a value.

#### Scenario: Verbose setup output

- **WHEN** the user runs `spry setup --verbose` with a valid recipe and org
- **THEN** Spry prints the resolved config path, name, org, and complete list, create when needed, and exec command lines

### Requirement: Empty setup provisions only

Zero setup commands is valid. After ensuring the VM exists (creating if needed, unless `--no-create`), Spry MUST exit success without running `sprite exec`.

#### Scenario: Empty setup creates missing VM

- **WHEN** the recipe has a name, no setup commands, and the VM does not exist
- **THEN** Spry creates the VM and exits success without exec

#### Scenario: Empty setup on existing VM

- **WHEN** the recipe has a name, no setup commands, and the VM exists
- **THEN** Spry exits success without create or exec

### Requirement: Git is not required

`spry setup` MUST NOT require the working directory to be a git repository.

#### Scenario: Setup outside git

- **WHEN** the user runs `spry setup` in a directory that is not a git repository and a valid recipe is found
- **THEN** Spry proceeds with sprite list/create/exec and does not fail for lack of git

### Requirement: Templated sprite name before provision

After loading the recipe and applying `--sprite` / `--org` / `--branch`, `spry setup` MUST expand placeholders in the resolved sprite name before `sprite list` or `sprite create`. `--sprite` MUST be expanded the same way as recipe `name`.

`--verbose` MUST print the resolved template values for branch, branch_slug, commit, and remote (each the value or `absent`).

#### Scenario: Unique VM name per branch

- **WHEN** the recipe has `name: myapp-{{branch_slug}}` and cwd is on branch `feature/add-dashboard`
- **THEN** Spry uses sprite name `myapp-feature-add-dashboard` for list/create/exec

#### Scenario: Sprite flag is expanded

- **WHEN** the user passes `--sprite 'app-{{branch_slug}}'` and `--branch feat/x`
- **THEN** the resolved sprite name is `app-feat-x`

#### Scenario: Verbose prints template context

- **WHEN** the user runs `spry setup --verbose` from a git checkout with origin
- **THEN** output includes the resolved branch, branch_slug, commit, and remote (or `absent` for any missing field)

### Requirement: Templated setup commands at exec time

Each setup command MUST be expanded immediately before `sprite exec` or before it is printed in `--dry-run`. Dry-run output MUST show the expanded command, not the raw `{{placeholders}}`.

#### Scenario: Dry-run shows expanded git checkout

- **WHEN** a setup command is `git checkout "{{branch}}"` and `--branch feat/x` is set
- **THEN** dry-run prints a `sprite exec` line containing `git checkout "feat/x"` and does not print `{{branch}}` as the checkout argument

#### Scenario: Checkout uses real branch not slug

- **WHEN** a setup command is `git checkout "{{branch}}"` on branch `feature/add-dashboard`
- **THEN** the executed command contains `feature/add-dashboard` (with the slash)

### Requirement: Setup expands sprite and org placeholders

After the VM name is resolved, each setup command (sprite or host) MUST expand `{{sprite}}` to that name and `{{org}}` to the resolved org when present.

#### Scenario: Host command uses sprite placeholder

- **WHEN** a host step is `sprite exec -s {{sprite}} -o {{org}} -- true` with name `demo` and org `acme`
- **THEN** the host command run is `sprite exec -s demo -o acme -- true`
