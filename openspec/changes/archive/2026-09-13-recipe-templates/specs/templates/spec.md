## ADDED Requirements

### Requirement: Placeholder set

Spry SHALL expand the following placeholders in the resolved sprite name and in each setup command string:

- `{{branch}}`: current git branch name
- `{{branch_slug}}`: slug of that branch name (see slug requirement)
- `{{commit}}`: full git commit SHA of HEAD
- `{{remote}}`: URL of the `origin` remote

Placeholders MUST be case-sensitive and MUST NOT allow interior whitespace (`{{ branch }}` is unknown). Expansion MUST be a single raw substitution (not recursive). Values MUST NOT be shell-quoted by Spry.

Unknown placeholders MUST cause a failure that names the placeholder.

#### Scenario: All known placeholders expand

- **WHEN** git reports branch `feature/add-dashboard`, commit `abc123def`, and origin `git@github.com:example-org/example-app.git`
- **THEN** `{{branch}}` is `feature/add-dashboard`, `{{branch_slug}}` is `feature-add-dashboard`, `{{commit}}` is `abc123def`, and `{{remote}}` is `git@github.com:example-org/example-app.git`

#### Scenario: Unknown placeholder fails

- **WHEN** a string contains `{{foo}}`
- **THEN** Spry fails and the error names `foo`

#### Scenario: Interior whitespace is unknown

- **WHEN** a string contains `{{ branch }}`
- **THEN** Spry fails as an unknown placeholder

#### Scenario: Values are not re-expanded

- **WHEN** `{{branch}}` resolves to the literal text `{{remote}}`
- **THEN** the output contains `{{remote}}` and is not expanded again

### Requirement: Git discovery from cwd

Spry SHALL read git metadata from the current working directory of `spry setup` (not from the recipe file path). `--config` MUST NOT change the git root.

`--branch <name>` MUST supply `{{branch}}` and `{{branch_slug}}` even when git is missing. `{{commit}}` and `{{remote}}` MUST still come from git.

A missing git binary, a non-git directory, or a failed git command MUST treat the corresponding field as absent rather than failing until a placeholder needs it.

Detached HEAD (`git rev-parse --abbrev-ref HEAD` is `HEAD`) MUST treat `{{branch}}` and `{{branch_slug}}` as absent unless `--branch` is set. `{{commit}}` MUST still resolve.

#### Scenario: Git comes from cwd with explicit config

- **WHEN** the user passes `--config /tmp/recipe.yaml` and cwd is a git checkout on branch `feat`
- **THEN** `{{branch}}` is `feat`

#### Scenario: Branch flag without git

- **WHEN** cwd is not a git repository and the user passes `--branch feat/x`
- **THEN** `{{branch}}` is `feat/x` and `{{branch_slug}}` is `feat-x`

#### Scenario: Commit still needs git when branch is overridden

- **WHEN** cwd is not a git repository, `--branch feat` is set, and a string contains `{{commit}}`
- **THEN** Spry fails because commit is missing

#### Scenario: Detached HEAD has commit but no branch

- **WHEN** `git rev-parse --abbrev-ref HEAD` is `HEAD` and a full SHA exists
- **THEN** `{{commit}}` expands and `{{branch}}` is treated as missing

#### Scenario: No origin remote

- **WHEN** the repository has no `origin` remote and a string contains `{{remote}}`
- **THEN** Spry fails because remote is missing

### Requirement: Missing placeholder fails that value

If the resolved **name** contains a placeholder that cannot be resolved, Spry MUST fail before listing or creating a sprite. The error MUST name the placeholder and MUST include a “To fix this” hint (`--branch` for branch/slug; run from a git checkout for commit/remote).

If a **setup command** contains a placeholder that cannot be resolved, Spry MUST fail that command without running it. Earlier setup commands MAY already have run. Dry-run MUST fail at the same point instead of printing that line as an action.

Recipes and flags with no placeholders MUST NOT require git.

#### Scenario: Name needs branch and git is missing

- **WHEN** the recipe `name` is `app-{{branch_slug}}`, cwd is not a git repository, and `--branch` is not set
- **THEN** Spry fails before `sprite list` and the error mentions `branch_slug` or `--branch`

#### Scenario: Setup line needs remote after provision

- **WHEN** the resolved name has no placeholders, the VM exists, setup command 1 has no placeholders, and command 2 contains `{{remote}}` with no origin
- **THEN** Spry runs command 1, does not run command 2, and fails naming `remote`

#### Scenario: No placeholders does not require git

- **WHEN** name is `demo` and setup is `echo hi` in a directory that is not a git repository
- **THEN** Spry proceeds to list/create/exec without failing for lack of git

### Requirement: Branch slug for sprite names

`{{branch_slug}}` MUST be derived from the resolved branch (`--branch` or git) by replacing each run of characters outside `[A-Za-z0-9._-]` with a single `-`, then stripping leading and trailing `-`. Case MUST be preserved.

If the expanded sprite name still contains `/`, Spry MUST fail and tell the user to use `{{branch_slug}}`.

An empty slug after stripping MUST be treated as a missing branch slug.

#### Scenario: Slashed branch becomes hyphenated

- **WHEN** the branch is `feature/add-dashboard`
- **THEN** `{{branch_slug}}` is `feature-add-dashboard`

#### Scenario: Name using raw branch with slash fails

- **WHEN** the expanded name is `feature/add-dashboard`
- **THEN** Spry fails and the error mentions `{{branch_slug}}`

#### Scenario: Adjacent separators collapse

- **WHEN** the branch is `feat//x y`
- **THEN** `{{branch_slug}}` is `feat-x-y`
