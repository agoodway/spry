## MODIFIED Requirements

### Requirement: Starter recipe comments

The starter recipe written by `spry init` MUST include commented examples for `start:`, `stop:`, and `teardown:` lists.

#### Scenario: Init comments mention start stop and teardown

- **WHEN** the user runs `spry init` in an empty directory
- **THEN** the written file contains commented `start:`, `stop:`, and `teardown:` examples, and the parsed `start`, `stop`, and `teardown` lists are empty
