## MODIFIED Requirements

### Requirement: Write a starter recipe

`spry init` SHALL write valid TOML to `.spry.toml` in cwd unless `--output` is given. It MUST set `sprite.name` to the literal current directory basename and `lifecycle.setup` to an empty array. It MUST NOT require Sprite or Git, expand placeholders, or supply an identifier. On success, it MUST print the created file's path.

#### Scenario: Default init

- **WHEN** init runs in a directory named `demo` without `.spry.toml`
- **THEN** it creates `.spry.toml` with `sprite.name` equal to `demo` and empty setup, and prints its path

#### Scenario: Init does not need sprite CLI

- **WHEN** init runs without the `sprite` executable available
- **THEN** it still writes the recipe

#### Scenario: Init outside git

- **WHEN** init runs outside a Git repository with no Git executable available
- **THEN** it still writes the recipe

#### Scenario: Basename is correctly encoded as TOML

- **WHEN** the directory basename contains quotes, backslashes, or whitespace
- **THEN** loading the generated TOML yields exactly that basename as `sprite.name`

### Requirement: Refuse to overwrite without force

If the target exists and `--force` is absent, init MUST fail without modifying it and mention `--force`. With `--force`, init MUST replace its target. Init MUST NOT automatically modify or delete neighboring legacy YAML recipes.

#### Scenario: Existing file without force

- **WHEN** `.spry.toml` exists and init runs without force
- **THEN** Spry leaves it unchanged and reports the overwrite refusal

#### Scenario: Existing file with force

- **WHEN** `.spry.toml` exists and init runs with `--force`
- **THEN** Spry replaces that TOML file

#### Scenario: Legacy file is preserved

- **WHEN** `.spry.yaml` exists and init writes `.spry.toml`
- **THEN** the YAML remains unchanged and a stderr migration note warns that TOML takes precedence and the VM name and cleanup steps need verification

### Requirement: Custom output path

`--output <path>` SHALL select the starter target instead of `.spry.toml`, resolving relative paths against cwd and retaining overwrite protection. Existing behavior for absolute paths and creating missing parent directories MUST remain. Output paths ending in `.yaml` or `.yml` MUST be rejected with a TOML-path hint, without modifying the target even with `--force`.

#### Scenario: Init with output path

- **WHEN** init runs with `--output custom.toml` and the file is absent
- **THEN** Spry writes TOML there and does not create `.spry.toml`

#### Scenario: Output path exists without force

- **WHEN** `custom.toml` exists and init runs with that output but no force
- **THEN** Spry fails and leaves it unchanged

#### Scenario: Nested absolute output

- **WHEN** init receives an absolute output path with missing parent directories
- **THEN** it creates the parents and writes a loadable TOML recipe at that path

#### Scenario: YAML output is rejected

- **WHEN** init receives `--output custom.yaml --force`
- **THEN** it reports that TOML output is required and does not create or overwrite that path

### Requirement: Starter recipe comments

The starter MUST include commented examples for an identifier-based `sprite.name`, optional org, lifecycle setup/teardown, and app start/stop. It MUST NOT advertise removed Git placeholders or resume hooks. Examples MUST NOT populate active start/stop/teardown lists or make identifier mandatory for the fixed-name default.

#### Scenario: Init comments mention templates

- **WHEN** init runs in a directory named `demo`
- **THEN** its text contains a commented `demo-{{identifier}}` name example, while the parsed name remains `demo`

#### Scenario: Init comments mention start stop and teardown

- **WHEN** init writes a starter
- **THEN** it contains commented app start/stop and lifecycle teardown examples, with empty parsed lists

#### Scenario: Init still does not need git

- **WHEN** init runs outside a Git repository
- **THEN** it writes a recipe without resolving any Git placeholders

## ADDED Requirements

### Requirement: Init explains neighboring legacy recipes

After successfully writing TOML beside `.spry.yaml` or `spry.yaml`, init MUST emit a stderr migration note naming the output and all neighboring legacy files. It MUST explain TOML discovery precedence and tell the operator to verify VM identity and cleanup steps. This MUST apply to custom output directories and force writes, without modifying YAML. Failed writes MUST NOT emit a successful migration note.

#### Scenario: Default output beside legacy YAML

- **WHEN** init successfully creates `.spry.toml` beside `.spry.yaml`
- **THEN** stderr names both paths and explains precedence and the need to check the name and teardown commands

#### Scenario: Custom forced output checks its directory

- **WHEN** init successfully overwrites `other/recipe.toml` with `--force` and `other/spry.yaml` exists
- **THEN** the migration note names those files, warns that normal discovery prefers TOML over YAML when both default recipe formats are present, and leaves YAML unchanged

#### Scenario: Failed write has no success note

- **WHEN** init refuses to overwrite a target beside legacy YAML without force
- **THEN** it reports the overwrite error without claiming that a new TOML recipe was written
