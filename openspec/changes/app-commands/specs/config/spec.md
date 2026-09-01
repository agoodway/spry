## ADDED Requirements

### Requirement: Recipe start list

The recipe MAY include `start`, a sequence of the same item types as `setup` (string or `{host: <string>}`). Omitted or empty `start` MUST be treated as zero commands. `start` MUST be independent of `stop` and `setup`.

#### Scenario: Start list parses

- **WHEN** a recipe contains `start` with an in-VM command and a host command
- **THEN** Spry loads both in file order and does not treat them as setup or stop commands
