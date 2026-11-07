## MODIFIED Requirements

### Requirement: Host setup steps

A `lifecycle.setup` item `{ host = "command" }` SHALL run on the operator's machine via `bash -lc`, not Sprite exec. Plain string items SHALL run inside the Sprite. Host commands MUST receive `SPRITE` as the resolved VM name and `ORG` when org is set. Dry-run MUST print expanded host commands prefixed with `host:` without execution.

#### Scenario: Host step is not sprite exec

- **WHEN** TOML `lifecycle.setup` contains `{ host = "gh repo deploy-key add" }` and setup runs with dry-run
- **THEN** output includes `host: gh repo deploy-key add` and does not execute that step through Sprite exec or the host shell

#### Scenario: Host env includes sprite name

- **WHEN** resolved name is `demo`, org is `acme`, and a host step runs
- **THEN** its environment includes `SPRITE=demo` and `ORG=acme`
