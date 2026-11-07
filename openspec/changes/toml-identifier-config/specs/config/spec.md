## ADDED Requirements

### Requirement: Structured TOML recipe schema

Spry SHALL load TOML with optional tables `[sprite]` (`name`, `org` strings), `[lifecycle]` (`setup`, `teardown` arrays), and `[app]` (`start`, `stop` arrays). Each item MUST be a VM command string or a host-command table with exactly one string field `host`. Unknown root, table, and step fields MUST fail with the file path. Empty name/org MUST be absent; omitted tables/lists MUST yield zero commands. Parse failures MUST identify the file and TOML parsing error.

#### Scenario: Valid recipe with all fields

- **WHEN** TOML contains sprite name/org and distinct lifecycle setup/teardown and app start/stop arrays
- **THEN** Spry loads each value and keeps each array in file order without mixing phases

#### Scenario: Valid recipe with only name

- **WHEN** a recipe contains only `[sprite]` with `name = "demo"`
- **THEN** Spry loads name `demo`, no org, and zero commands for every phase

#### Scenario: Unknown keys are rejected

- **WHEN** a recipe contains an unknown root key, an unknown table field such as `lifecycle.resume`, or an extra field in a host step
- **THEN** Spry fails with a file-specific error rather than ignoring that field

#### Scenario: Legacy unknown key is rejected

- **WHEN** a recipe migrated from the formerly permissive schema retains an unknown `color` field
- **THEN** Spry now rejects the field with a file-specific error; the former ignore behavior is intentionally removed

#### Scenario: Empty name is absent

- **WHEN** a recipe contains `[sprite]` with `name = ""` and `org = ""`
- **THEN** Spry treats both values as absent

#### Scenario: Invalid TOML

- **WHEN** a recipe is not valid TOML
- **THEN** Spry fails with an error naming the file and explaining the TOML parse failure

#### Scenario: Invalid YAML

- **WHEN** an explicitly selected legacy `.yaml` recipe contains invalid YAML
- **THEN** Spry reports that YAML recipes must be migrated, without attempting YAML parsing or execution

#### Scenario: Multiline shell command

- **WHEN** a command uses a TOML multiline literal string containing newlines, shell quotes, and dollar signs
- **THEN** Spry loads the string using TOML string semantics without adding shell escaping or changing command order

### Requirement: Actionable recipe diagnostics

Recipe errors MUST identify the file. Invalid steps MUST identify the phase/index, offending field or type, and expected string or `{ host = "..." }` shape instead of only a generic enum mismatch. Legacy root identity and phase keys MUST suggest their new TOML tables.

#### Scenario: Invalid step identifies the field

- **WHEN** `lifecycle.setup` item 2 is `{ host = "echo hi", typo = true }`
- **THEN** the error identifies the file, setup item 2, unsupported `typo` field, and expected step shape

#### Scenario: Legacy root keys suggest the correct tables

- **WHEN** a TOML recipe contains old root-level `name`, `org`, `setup`, `teardown`, `start`, or `stop` fields
- **THEN** the error suggests `[sprite]`, `[lifecycle]`, or `[app]` as appropriate

### Requirement: Warn when TOML shadows legacy YAML

When selected TOML has `.spry.yaml` or `spry.yaml` in the same directory, Spry MUST warn once per invocation on stderr before VM operations, naming the TOML and all ignored legacy files. This MUST apply to discovered and explicit configs, including dry-run. TOML MUST remain authoritative; YAML MUST NOT be parsed, executed, or modified. Legacy files in unselected directories MUST NOT trigger warnings.

#### Scenario: Selected TOML shadows both legacy filenames

- **WHEN** a discovered TOML recipe sits beside `.spry.yaml` and `spry.yaml`
- **THEN** stderr names the selected TOML and both ignored files once before VM operations, and only TOML supplies the identity and commands

#### Scenario: Explicit dry-run warns without mutation

- **WHEN** an explicitly selected TOML has neighboring legacy YAML and dry-run is requested
- **THEN** the same warning appears on stderr, and neither recipe file nor VM state is mutated

#### Scenario: Unselected ancestor does not warn

- **WHEN** a child TOML is selected and only an ancestor has legacy YAML
- **THEN** no YAML-shadowing warning is emitted

## MODIFIED Requirements

### Requirement: Walk-up config discovery

Without `--config`, Spry SHALL search from cwd toward the filesystem root, checking `.spry.toml` then `spry.toml` in each directory. The nearest recipe MUST win, with dotted TOML preferred in the same directory. At a directory containing legacy `.spry.yaml` or `spry.yaml` but no TOML recipe, discovery MUST stop with migration guidance naming that file. Discovery MUST NOT depend on Git or load a separate home-config location.

#### Scenario: Config in current directory

- **WHEN** cwd contains `.spry.toml`
- **THEN** Spry loads that file

#### Scenario: Prefers dotted name in the same directory

- **WHEN** cwd contains both `.spry.toml` and `spry.toml`
- **THEN** Spry loads `.spry.toml`

#### Scenario: Walks up to a parent

- **WHEN** cwd has no TOML or legacy recipe and a parent contains `.spry.toml`
- **THEN** Spry loads the parent's recipe

#### Scenario: Child wins over parent

- **WHEN** cwd contains `spry.toml` and a parent contains `.spry.toml`
- **THEN** Spry loads cwd's `spry.toml`

#### Scenario: No recipe found

- **WHEN** no TOML or legacy recipe exists from cwd to the filesystem root
- **THEN** Spry reports missing `.spry.toml` / `spry.toml` and suggests `spry init` or `--config`

#### Scenario: Legacy child does not select ancestor VM

- **WHEN** cwd contains `.spry.yaml` without TOML and a parent contains `.spry.toml`
- **THEN** Spry fails with guidance to migrate cwd's YAML and does not select the parent recipe

#### Scenario: TOML wins over legacy YAML in the same directory

- **WHEN** cwd contains `spry.toml` and `.spry.yaml`
- **THEN** Spry loads `spry.toml`, warns on stderr that the YAML is ignored, and does not execute or modify YAML

### Requirement: Explicit config path

With `--config <path>`, Spry MUST use only that path, resolving relative paths against cwd. Missing files MUST fail with the resolved path. Existing paths ending in `.yaml` or `.yml` MUST fail with TOML migration guidance. Other paths MUST be parsed as TOML, without requiring a `.toml` suffix or falling back to YAML.

#### Scenario: Custom config path

- **WHEN** the user passes `--config /tmp/recipe.toml` and that file exists
- **THEN** Spry loads it even if cwd contains another recipe

#### Scenario: Missing custom config path

- **WHEN** the user passes `--config missing.toml` and it does not exist
- **THEN** Spry fails and includes the resolved path

#### Scenario: Legacy explicit config

- **WHEN** the user passes an existing `.yaml` or `.yml` config path
- **THEN** Spry fails with migration guidance and does not execute its commands

#### Scenario: Extensionless TOML config

- **WHEN** the user passes `--config ./recipe` and the file contains valid TOML
- **THEN** Spry parses that file as TOML

### Requirement: Flag overrides for name and org

`--sprite` MUST override `sprite.name`; `--org` MUST override `sprite.org`. A flag MUST NOT be required when the recipe supplies its value. Overrides MUST be applied before name placeholder expansion.

#### Scenario: Flag overrides name

- **WHEN** `sprite.name` is `from-file` and the user passes `--sprite from-flag`
- **THEN** the resolved sprite name is `from-flag`

#### Scenario: Flag overrides org

- **WHEN** `sprite.org` is `file-org` and the user passes `--org flag-org`
- **THEN** the resolved org is `flag-org`

#### Scenario: File values used when flags omitted

- **WHEN** TOML provides `sprite.name` and `sprite.org` and neither flag is passed
- **THEN** Spry uses those file values

#### Scenario: Fixed override bypasses unused name placeholder

- **WHEN** `sprite.name` is `app-{{identifier}}`, `--sprite fixed` is passed, and the selected phase does not reference identifier
- **THEN** Spry targets `fixed` without requiring `--identifier`

### Requirement: Setup items may be host maps

`lifecycle.setup` SHALL accept VM command strings and inline tables `{ host = "command" }`. Invalid types, empty tables, and extra or unknown step fields MUST fail TOML recipe parsing.

#### Scenario: Mixed setup list

- **WHEN** setup contains `"echo in-vm"` followed by `{ host = "echo on-host" }`
- **THEN** Spry loads one VM step followed by one host step

#### Scenario: Invalid host table

- **WHEN** setup contains `{ host = "echo hi", typo = true }` or `{ host = 42 }`
- **THEN** Spry rejects the recipe before any command runs

### Requirement: Recipe stop list

`app.stop` SHALL accept the same step types as `lifecycle.setup`. An omitted or empty list MUST be zero commands and MUST remain independent of all other phases.

#### Scenario: Stop list parses

- **WHEN** `app.stop` contains a VM string and a host table
- **THEN** Spry loads both in order as stop commands only

### Requirement: Recipe start list

`app.start` SHALL accept the same step types as `lifecycle.setup`. An omitted or empty list MUST be zero commands and MUST remain independent of all other phases.

#### Scenario: Start list parses

- **WHEN** `app.start` contains a VM string and a host table
- **THEN** Spry loads both in order as start commands only

### Requirement: Recipe teardown list

`lifecycle.teardown` SHALL accept the same step types as `lifecycle.setup`. An omitted or empty list MUST be zero commands and MUST remain independent of setup, app start, and app stop.

#### Scenario: Teardown list parses

- **WHEN** `lifecycle.teardown` contains a VM string and a host table
- **THEN** Spry loads both in order as teardown commands only

## REMOVED Requirements

### Requirement: Recipe file schema

**Reason**: The permissive flat YAML schema is replaced by the strict structured TOML recipe schema.

**Migration**: Use `[sprite]` for name/org, `[lifecycle]` for setup/teardown, and `[app]` for start/stop, with TOML host tables. Remove unknown fields rather than relying on them being ignored.
