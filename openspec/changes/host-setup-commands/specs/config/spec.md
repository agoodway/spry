## ADDED Requirements

### Requirement: Setup items may be host maps

The recipe `setup` list SHALL accept a string (sprite command) or a mapping `{host: <string>}` (host command). Unknown mapping shapes MUST fail YAML parse.

#### Scenario: Mixed setup list

- **WHEN** setup is `- echo in-vm` then `- host: gh repo deploy-key add`
- **THEN** Spry loads a sprite step and a host step in that order
