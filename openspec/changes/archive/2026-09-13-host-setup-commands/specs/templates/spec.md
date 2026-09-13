## ADDED Requirements

### Requirement: Sprite and org placeholders in setup

`{{sprite}}` and `{{org}}` SHALL expand in setup commands using the resolved VM name and org. They MUST NOT be required when expanding the recipe `name` field.

#### Scenario: Sprite placeholder in setup

- **WHEN** a setup string contains `{{sprite}}` and the resolved name is `myapp-feat`
- **THEN** expansion yields `myapp-feat`
