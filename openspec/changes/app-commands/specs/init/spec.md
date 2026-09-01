## MODIFIED Requirements

### Requirement: Starter recipe comments

The starter recipe written by `spry init` MUST include commented examples for both `start:` and `stop:` lists.

#### Scenario: Init comments mention start and stop

- **WHEN** the user runs `spry init` in an empty directory
- **THEN** the written file contains commented `start:` and `stop:` examples, and the parsed `start` and `stop` lists are empty
