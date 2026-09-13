## ADDED Requirements

### Requirement: Setup expands sprite and org placeholders

After the VM name is resolved, each setup command (sprite or host) MUST expand `{{sprite}}` to that name and `{{org}}` to the resolved org when present.

#### Scenario: Host command uses sprite placeholder

- **WHEN** a host step is `sprite exec -s {{sprite}} -o {{org}} -- true` with name `demo` and org `acme`
- **THEN** the host command run is `sprite exec -s demo -o acme -- true`
