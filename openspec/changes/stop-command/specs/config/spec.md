## ADDED Requirements

### Requirement: Recipe stop list

The recipe MAY include `stop`, a sequence of the same item types as `setup` (string or `{host: <string>}`). Omitted or empty `stop` MUST be treated as zero commands.

#### Scenario: Stop list parses

- **WHEN** a recipe contains `stop` with an in-VM command and a host command
- **THEN** Spry loads both in file order and does not treat them as setup commands
