## ADDED Requirements

### Requirement: Starter recipe comments document templates

`spry init` MUST still write a valid recipe whose `name` is the current directory basename and whose `setup` is an empty list. It MUST NOT require git and MUST NOT expand placeholders.

The starter file MUST include comments that show `{{branch_slug}}` in a `name` example and `{{remote}}` / `{{branch}}` in setup examples.

#### Scenario: Init comments mention templates

- **WHEN** the user runs `spry init` in a directory named `demo`
- **THEN** the created file parses with name `demo` and empty setup, and its text contains `{{branch_slug}}`, `{{remote}}`, and `{{branch}}`

#### Scenario: Init still does not need git

- **WHEN** the user runs `spry init` in a directory that is not a git repository
- **THEN** Spry writes the recipe file
