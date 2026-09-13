## ADDED Requirements

### Requirement: Recipe file schema

Spry SHALL load a YAML recipe with the following fields:

- `name` (string, optional in the file): sprite VM name
- `org` (string, optional): sprite organization
- `setup` (sequence of strings, optional): shell commands to run inside the VM

Unknown keys MUST be ignored. An empty string `name` MUST be treated as absent. An empty string `org` MUST be treated as absent. An omitted or empty `setup` list MUST be treated as zero commands.

#### Scenario: Valid recipe with all fields

- **WHEN** a recipe file contains `name`, `org`, and a `setup` list of two commands
- **THEN** Spry loads that name, org, and both commands in file order

#### Scenario: Valid recipe with only name

- **WHEN** a recipe file contains only `name: demo`
- **THEN** Spry loads name `demo`, no org, and zero setup commands

#### Scenario: Unknown keys are ignored

- **WHEN** a recipe file contains `name` plus an unknown key `color`
- **THEN** Spry loads the name and does not fail because of `color`

#### Scenario: Empty name is absent

- **WHEN** a recipe file contains `name: ""`
- **THEN** Spry treats the name as missing

#### Scenario: Invalid YAML

- **WHEN** a recipe file is not valid YAML
- **THEN** Spry fails with an error that names the file and says the YAML could not be parsed

### Requirement: Walk-up config discovery

When no `--config` path is given, Spry SHALL search for a recipe starting at the current working directory and walking toward the filesystem root. In each directory it MUST look for `.spry.yaml` first, then `spry.yaml`. The first file found MUST be used.

Spry MUST NOT require a git repository. Spry MUST NOT load config from the user’s home config directory.

#### Scenario: Config in current directory

- **WHEN** the current directory contains `.spry.yaml`
- **THEN** Spry loads that file

#### Scenario: Prefers dotted name in the same directory

- **WHEN** the current directory contains both `.spry.yaml` and `spry.yaml`
- **THEN** Spry loads `.spry.yaml`

#### Scenario: Walks up to a parent

- **WHEN** the current directory has no recipe and a parent directory contains `.spry.yaml`
- **THEN** Spry loads the parent’s `.spry.yaml`

#### Scenario: Child wins over parent

- **WHEN** both the current directory and a parent contain a recipe file
- **THEN** Spry loads the current directory’s file

#### Scenario: No recipe found

- **WHEN** no `.spry.yaml` or `spry.yaml` exists from cwd to filesystem root
- **THEN** Spry fails with an error that explains how to run `spry init` or pass `--config`

### Requirement: Explicit config path

When `--config <path>` is provided, Spry MUST load that path and MUST NOT walk the directory tree. A relative path MUST be resolved against the current working directory. If the file does not exist, Spry MUST fail with an error that includes the resolved path.

#### Scenario: Custom config path

- **WHEN** the user passes `--config /tmp/recipe.yaml` and that file exists
- **THEN** Spry loads `/tmp/recipe.yaml` even if a `.spry.yaml` exists in cwd

#### Scenario: Missing custom config path

- **WHEN** the user passes `--config missing.yaml` and the file does not exist
- **THEN** Spry fails and the error includes the resolved path

### Requirement: Flag overrides for name and org

`--sprite` MUST override the recipe `name`. `--org` MUST override the recipe `org`. A flag MUST NOT be required when the recipe already supplies the value.

#### Scenario: Flag overrides name

- **WHEN** the recipe has `name: from-file` and the user passes `--sprite from-flag`
- **THEN** the resolved sprite name is `from-flag`

#### Scenario: Flag overrides org

- **WHEN** the recipe has `org: file-org` and the user passes `--org flag-org`
- **THEN** the resolved org is `flag-org`

#### Scenario: File values used when flags omitted

- **WHEN** the recipe has `name: from-file` and `org: file-org` and neither `--sprite` nor `--org` is passed
- **THEN** the resolved name is `from-file` and the resolved org is `file-org`
