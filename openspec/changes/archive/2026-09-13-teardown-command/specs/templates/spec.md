## ADDED Requirements

### Requirement: Placeholders expand in teardown commands

Placeholders SHALL expand in each teardown command the same way as in setup commands (`{{branch}}`, `{{branch_slug}}`, `{{commit}}`, `{{remote}}`, `{{sprite}}`, `{{org}}`). Expansion MUST happen immediately before exec/host run or before the line is printed in `--dry-run`. Dry-run output MUST show the expanded command, not the raw placeholders.

A teardown command with an unresolvable placeholder MUST fail that command without running it and MUST NOT destroy. Earlier teardown commands MAY already have run.

#### Scenario: Dry-run shows expanded teardown host command

- **WHEN** a teardown host command contains `{{sprite}}` and the resolved name is `demo`
- **THEN** dry-run prints `host:` with `demo` and does not print `{{sprite}}`

#### Scenario: Teardown line needs remote after earlier step

- **WHEN** the VM exists, teardown command 1 has no placeholders, command 2 contains `{{remote}}` with no origin
- **THEN** Spry runs command 1, does not run command 2, does not destroy, and fails naming `remote`
