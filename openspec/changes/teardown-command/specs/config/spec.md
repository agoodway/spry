## ADDED Requirements

### Requirement: Recipe teardown list

The recipe MAY include `teardown`, a sequence of the same item types as `setup` (string or `{host: <string>}`). Omitted or empty `teardown` MUST be treated as zero commands. `teardown` MUST be independent of `setup`, `start`, and `stop`.

#### Scenario: Teardown list parses

- **WHEN** a recipe contains `teardown` with an in-VM command and a host command
- **THEN** Spry loads both in file order and does not treat them as setup, start, or stop commands
