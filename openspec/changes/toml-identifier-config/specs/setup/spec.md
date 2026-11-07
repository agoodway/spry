## MODIFIED Requirements

### Requirement: Sprite name is required

Setup MUST resolve a name from `--sprite` or TOML `sprite.name`. If neither is present, it MUST fail with a hint to set `sprite.name` or pass `--sprite`. Org is optional; when absent, Sprite invocations MUST omit `-o`.

#### Scenario: Name missing

- **WHEN** the recipe has no sprite name and no `--sprite` is supplied
- **THEN** Spry fails with guidance mentioning `sprite.name` and `--sprite`

#### Scenario: Org omitted from invocations

- **WHEN** no org is resolved
- **THEN** list, create, and exec omit `-o`

### Requirement: Run setup commands in order and fail fast

Setup SHALL run `lifecycle.setup` in list order. VM strings MUST execute as `sprite exec -s <name> [-o <org>] -- sh -c <command>`; host tables MUST execute on the host. Each step MUST be labeled `[i/n]`, display output, and stop later steps on failure with a setup/index error. Every explicit setup invocation MUST rerun the full setup list after ensuring the VM exists. Setup MUST NOT implicitly run app start/stop or teardown.

#### Scenario: All commands succeed

- **WHEN** three setup commands succeed
- **THEN** Spry labels and executes all three in order

#### Scenario: Middle command fails

- **WHEN** setup command 2 of 3 fails
- **THEN** command 3 does not run and the error identifies setup command 2 of 3

#### Scenario: Exec includes org when set

- **WHEN** org is `acme` and a VM setup command runs
- **THEN** its exec invocation includes the resolved name and `-o acme`

#### Scenario: Existing VM setup is rerunnable

- **WHEN** setup is explicitly invoked twice for an existing VM with distinct setup and app start commands
- **THEN** both invocations run the entire setup list, neither creates the VM, and neither runs app start

## ADDED Requirements

### Requirement: Identifier-based sprite name before provision

After applying `--sprite` and `--org`, setup MUST expand `{{identifier}}` in the effective name before listing or creating. The configured name and `--sprite` MUST use the same expansion rules. Setup MUST accept `--identifier`, reject `--branch`, and report identifier (value or absent) in verbose output instead of Git metadata. Invalid identifiers and name expansion errors MUST fail before VM operations.

#### Scenario: Unique VM name per identifier

- **WHEN** `sprite.name` is `myapp-{{identifier}}` and identifier is `task-42`
- **THEN** all Sprite operations target `myapp-task-42`

#### Scenario: Removed branch_slug placeholder in name fails

- **WHEN** a migrated recipe still uses `myapp-{{branch_slug}}` as its name
- **THEN** setup fails with an unknown-placeholder error before VM operations rather than deriving a name from Git

#### Scenario: Sprite flag is expanded

- **WHEN** the user passes `--sprite 'other-{{identifier}}' --identifier task-42`
- **THEN** the resolved name is `other-task-42`

#### Scenario: Verbose prints template context

- **WHEN** setup runs with `--verbose --identifier task-42`
- **THEN** output reports that identifier, config path, resolved name/org, and complete Sprite invocation lines without Git metadata

### Requirement: Setup command expansion at execution

Each setup command MUST expand supported placeholders immediately before VM/host execution or dry-run printing. Dry-run MUST show expanded commands. An expansion failure MUST stop that command and later commands, while earlier commands can already have run.

#### Scenario: Dry-run shows expanded identifier

- **WHEN** setup contains `echo '{{identifier}}'` and identifier is `task-42`
- **THEN** dry-run shows `echo 'task-42'` without executing it

#### Scenario: Identifier is not rewritten

- **WHEN** setup references identifier and `--identifier Task_42-a` is supplied
- **THEN** the command receives exactly `Task_42-a`, preserving case and separators

#### Scenario: Literal checkout passes through dry-run unchanged

- **WHEN** setup contains the literal command `git checkout 'feature/add-dashboard'` and identifier is `task-42`
- **THEN** dry-run prints that checkout unchanged; Spry does not discover or substitute a Git branch

#### Scenario: Explicit checkout branch is independent of identifier

- **WHEN** a setup script explicitly checks out `feature/add-dashboard` while identifier is `task-42`
- **THEN** Spry passes the explicit checkout command unchanged rather than substituting the environment identifier for the branch

## REMOVED Requirements

### Requirement: Templated sprite name before provision

**Reason**: Git-derived names and verbose Git context are replaced by explicit identifier-based name resolution.

**Migration**: Use `sprite.name = "app-{{identifier}}"` or a literal name, pass `--identifier` when referenced, and replace removed Git placeholders before setup.

### Requirement: Templated setup commands at exec time

**Reason**: The Git-oriented expansion contract is replaced by execution-time expansion of identifier/sprite/org only.

**Migration**: Keep Git operations explicit in commands or scripts. Supported placeholders still expand immediately before each command; dry-run prints the same expanded commands without executing them.
