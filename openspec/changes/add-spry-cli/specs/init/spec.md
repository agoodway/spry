## ADDED Requirements

### Requirement: Write a starter recipe

`spry init` SHALL write a YAML recipe to `.spry.yaml` in the current working directory when `--output` is not given. The file MUST be valid YAML with `name` set to the current directory basename and `setup` set to an empty list (comments MAY document examples). `spry init` MUST NOT require the `sprite` CLI and MUST NOT require a git repository.

On success, Spry MUST print the path of the file it created.

#### Scenario: Default init

- **WHEN** the user runs `spry init` in a directory named `demo` with no `.spry.yaml`
- **THEN** Spry creates `.spry.yaml` in that directory with `name: demo` and `setup: []`, and prints its path

#### Scenario: Init does not need sprite CLI

- **WHEN** the user runs `spry init` and `sprite` is not on PATH
- **THEN** Spry still writes the recipe file

#### Scenario: Init outside git

- **WHEN** the user runs `spry init` in a directory that is not a git repository
- **THEN** Spry still writes the recipe file

### Requirement: Refuse to overwrite without force

If the target file already exists and `--force` is not passed, `spry init` MUST fail without modifying the file. The error MUST mention `--force`.

When `--force` is passed, `spry init` MUST overwrite the target file.

#### Scenario: Existing file without force

- **WHEN** `.spry.yaml` already exists and the user runs `spry init`
- **THEN** Spry exits with an error, does not change the file, and mentions `--force`

#### Scenario: Existing file with force

- **WHEN** `.spry.yaml` already exists and the user runs `spry init --force`
- **THEN** Spry overwrites `.spry.yaml`

### Requirement: Custom output path

`--output <path>` SHALL write the starter recipe to that path instead of `.spry.yaml`. A relative path MUST be resolved against the current working directory. Overwrite protection (`--force`) MUST apply to this path the same way as the default filename.

#### Scenario: Init with output path

- **WHEN** the user runs `spry init --output custom.yaml` and `custom.yaml` does not exist
- **THEN** Spry writes `custom.yaml` and does not create `.spry.yaml`

#### Scenario: Output path exists without force

- **WHEN** `custom.yaml` exists and the user runs `spry init --output custom.yaml`
- **THEN** Spry exits with an error and does not change `custom.yaml`
