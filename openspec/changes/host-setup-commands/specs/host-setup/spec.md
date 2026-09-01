## ADDED Requirements

### Requirement: Host setup steps

A setup list item that is a mapping with key `host` SHALL run on the host via `bash -lc`, not via `sprite exec`. Plain strings SHALL still run inside the sprite.

Host commands MUST receive environment variable `SPRITE` set to the resolved VM name. When org is set, they MUST also receive `ORG`.

`--dry-run` MUST print the host command prefixed with `host:` and MUST NOT execute it.

#### Scenario: Host step is not sprite exec

- **WHEN** setup contains `host: gh repo deploy-key add` and the user runs `spry setup --dry-run`
- **THEN** output includes `host: gh repo deploy-key add` and does not run a `sprite exec` for that line

#### Scenario: Host env includes sprite name

- **WHEN** the resolved name is `demo`, org is `acme`, and a host step runs
- **THEN** the host command environment includes `SPRITE=demo` and `ORG=acme`
