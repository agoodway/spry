## ADDED Requirements

### Requirement: Templated sprite name before provision

After loading the recipe and applying `--sprite` / `--org` / `--branch`, `spry setup` MUST expand placeholders in the resolved sprite name before `sprite list` or `sprite create`. `--sprite` MUST be expanded the same way as recipe `name`.

`--verbose` MUST print the resolved template values for branch, branch_slug, commit, and remote (each the value or `absent`).

#### Scenario: Unique VM name per branch

- **WHEN** the recipe has `name: myapp-{{branch_slug}}` and cwd is on branch `feature/add-dashboard`
- **THEN** Spry uses sprite name `myapp-feature-add-dashboard` for list/create/exec

#### Scenario: Sprite flag is expanded

- **WHEN** the user passes `--sprite 'app-{{branch_slug}}'` and `--branch feat/x`
- **THEN** the resolved sprite name is `app-feat-x`

#### Scenario: Verbose prints template context

- **WHEN** the user runs `spry setup --verbose` from a git checkout with origin
- **THEN** output includes the resolved branch, branch_slug, commit, and remote (or `absent` for any missing field)

### Requirement: Templated setup commands at exec time

Each setup command MUST be expanded immediately before `sprite exec` or before it is printed in `--dry-run`. Dry-run output MUST show the expanded command, not the raw `{{placeholders}}`.

#### Scenario: Dry-run shows expanded git checkout

- **WHEN** a setup command is `git checkout "{{branch}}"` and `--branch feat/x` is set
- **THEN** dry-run prints a `sprite exec` line containing `git checkout "feat/x"` and does not print `{{branch}}` as the checkout argument

#### Scenario: Checkout uses real branch not slug

- **WHEN** a setup command is `git checkout "{{branch}}"` on branch `feature/add-dashboard`
- **THEN** the executed command contains `feature/add-dashboard` (with the slash)
