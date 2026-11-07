## MODIFIED Requirements

### Requirement: Placeholder set

Spry SHALL support only `{{identifier}}` in the effective sprite name, and `{{identifier}}`, `{{sprite}}`, and `{{org}}` in commands. Identifier comes from `--identifier`; sprite/org come from resolved config and overrides. Substitution MUST be case-sensitive, exact, nonrecursive, and not shell-escaped. Interior whitespace, unknown names, and unclosed placeholders MUST fail with actionable errors. Other config fields MUST NOT be templated.

#### Scenario: All known placeholders expand

- **WHEN** identifier is `task_42`, resolved sprite is `myapp-task_42`, org is `acme`, and a command references all three
- **THEN** expansion substitutes those three distinct values

#### Scenario: Unknown placeholder fails

- **WHEN** a string being expanded contains `{{foo}}`
- **THEN** Spry fails and names the unknown placeholder

#### Scenario: Removed Git placeholders fail

- **WHEN** a name or selected command being expanded contains `{{branch}}`, `{{branch_slug}}`, `{{commit}}`, or `{{remote}}`
- **THEN** Spry rejects it as unknown and lists the supported placeholders for that context

#### Scenario: Interior whitespace is unknown

- **WHEN** a string contains `{{ identifier }}`
- **THEN** Spry fails rather than trimming the placeholder name

#### Scenario: Values are not re-expanded

- **WHEN** resolved org is the literal `{{identifier}}` and a command contains `{{org}}`
- **THEN** expansion inserts literal `{{identifier}}` without another expansion pass

#### Scenario: Unclosed placeholder fails

- **WHEN** a string being expanded ends with `{{identifier`
- **THEN** Spry fails and suggests closing or removing the braces

#### Scenario: Org configuration is literal

- **WHEN** `sprite.org` contains braces and the user supplies no org override
- **THEN** Spry treats that configured value literally rather than expanding it

### Requirement: Sprite and org placeholders in setup

Setup commands SHALL expand `{{sprite}}` and `{{org}}` from the resolved VM name and org, after overrides and name expansion. They MUST NOT be available while expanding the name itself. `{{identifier}}` MUST remain the caller's label even when `--sprite` overrides the name.

#### Scenario: Sprite placeholder in setup

- **WHEN** setup contains `{{sprite}}` and resolved name is `myapp-task-42`
- **THEN** it expands to `myapp-task-42`

#### Scenario: Identifier and sprite are distinct

- **WHEN** the user passes `--identifier task-42 --sprite custom-vm` and setup references both placeholders
- **THEN** identifier expands to `task-42` and sprite expands to `custom-vm`

#### Scenario: No self-reference in names

- **WHEN** the effective name contains `{{sprite}}` or `{{org}}`
- **THEN** Spry fails before VM operations and explains that names support only `{{identifier}}`

## ADDED Requirements

### Requirement: Missing template values fail at execution boundary

An unresolvable placeholder in the effective name MUST fail before listing/provisioning. An unresolvable command placeholder MUST fail immediately before that command, without executing it; earlier steps can have run. Errors MUST name the value and supply a corrective hint, including `--identifier` for missing identifier. Dry-run MUST fail at the same expansion boundary. Unselected phases MUST NOT require placeholder values.

#### Scenario: Name needs identifier

- **WHEN** name is `app-{{identifier}}` and `--identifier` is absent
- **THEN** Spry fails before `sprite list` with a hint to pass `--identifier`

#### Scenario: Setup line needs identifier after provision

- **WHEN** name is fixed, setup command 1 needs no placeholders, command 2 uses `{{identifier}}`, and no identifier is supplied
- **THEN** Spry ensures the VM exists, runs command 1, and fails command 2 without executing it

#### Scenario: No placeholders does not require identifier

- **WHEN** effective name and selected phase have no placeholders and no identifier is supplied
- **THEN** Spry proceeds without requiring an identifier or Git

#### Scenario: Unused phase does not require identifier

- **WHEN** name is fixed, setup references identifier, app start does not, and the user runs app start without an identifier
- **THEN** app start succeeds on an existing VM without evaluating setup placeholders

#### Scenario: Missing org in a command

- **WHEN** a selected command contains `{{org}}` and no org is configured or supplied
- **THEN** Spry fails that command and suggests configuring org or passing `--org`

#### Scenario: Removed branch_slug fails without Git discovery

- **WHEN** the effective name contains the removed `{{branch_slug}}` placeholder and Git is unavailable
- **THEN** Spry reports an unknown placeholder before VM operations, without attempting Git discovery

#### Scenario: Removed remote placeholder fails after an earlier setup step

- **WHEN** setup command 1 has run and command 2 contains removed `{{remote}}`
- **THEN** Spry fails command 2 as unknown without executing it or later commands

#### Scenario: No placeholders does not require git

- **WHEN** name and setup commands are literal and neither Git nor identifier is available
- **THEN** Spry proceeds with Sprite operations without Git discovery

### Requirement: Teardown command template expansion

Teardown SHALL expand only identifier/sprite/org placeholders immediately before exec, host execution, or dry-run printing, as setup does. An unknown or unresolvable placeholder MUST prevent that step and VM destruction; earlier teardown commands can have run. Dry-run MUST print expanded commands.

#### Scenario: Dry-run shows expanded teardown host command

- **WHEN** a host teardown command contains `{{sprite}}` and resolved name is `demo`
- **THEN** dry-run prints `host:` with `demo` rather than the placeholder

#### Scenario: Teardown line needs identifier after earlier step

- **WHEN** the VM exists, teardown command 1 needs no placeholders, command 2 needs an absent identifier
- **THEN** Spry runs command 1, does not execute command 2, does not destroy, and names the missing identifier

#### Scenario: Removed remote placeholder prevents teardown destruction

- **WHEN** teardown command 1 has run and command 2 contains removed `{{remote}}`
- **THEN** Spry fails command 2 as unknown, does not execute it or later steps, and does not destroy

### Requirement: Explicit identifier validation

Setup, app start/stop, and teardown SHALL accept `--identifier`. Every value MUST start with an ASCII letter or digit and contain only ASCII letters, digits, `_`, and `-` thereafter. Invalid supplied values MUST fail before VM operations, even when unused. Case MUST be preserved without trimming or slugification. Spry MUST NOT generate identifiers, remember a previous value, or derive them from Git.

#### Scenario: Valid identifier is preserved

- **WHEN** the user passes `--identifier Task_42-a`
- **THEN** `{{identifier}}` expands exactly to `Task_42-a`

#### Scenario: Invalid identifier is rejected rather than rewritten

- **WHEN** a supplied identifier is empty or contains a slash, dot, whitespace, non-ASCII character, or shell metacharacter
- **THEN** Spry fails before VM operations and describes the allowed characters

#### Scenario: Unused invalid identifier still fails

- **WHEN** name is fixed and commands contain no placeholders, but `--identifier feat/x` is supplied
- **THEN** Spry rejects the invalid value before listing or executing

#### Scenario: Leading option-like identifiers are rejected

- **WHEN** `--identifier=-x` or `--identifier=_x` is supplied
- **THEN** Spry rejects it before VM operations and explains the alphanumeric-first rule

#### Scenario: One-character alphanumeric identifier is valid

- **WHEN** the supplied identifier is `7` or `A`
- **THEN** it is accepted and preserved unchanged

### Requirement: Existing dotted VM names remain addressable

Dots MUST remain invalid in identifiers but valid in otherwise accepted literal VM names and `--sprite` overrides. An override MUST replace the configured name before identifier expansion, allowing access to existing dotted VMs without changing identity or requiring an unused identifier.

#### Scenario: Teardown addresses a dotted legacy VM

- **WHEN** the recipe name is `myapp-{{identifier}}`, cleanup does not reference identifier, and teardown receives `--sprite myapp-release-1.2` without identifier
- **THEN** cleanup and destruction target exactly `myapp-release-1.2`, without creating or renaming a VM

#### Scenario: Literal dotted name remains valid

- **WHEN** `sprite.name` is the literal `myapp-release-1.2` and no identifier is supplied
- **THEN** Spry targets that existing name without identifier validation or rewriting the name

### Requirement: No implicit Git behavior

Spry SHALL resolve recipes and placeholders without invoking Git or inspecting checkout metadata. `--branch` MUST be rejected by setup, app start/stop, and teardown. Verbose output MUST report identifier as its supplied value or absent and MUST NOT report branch, branch slug, commit, or remote metadata. Recipe commands remain free to invoke Git explicitly.

#### Scenario: Git installation and checkout do not affect selection

- **WHEN** the same recipe and identifier are used in two directories with different Git metadata, or without Git installed
- **THEN** Spry resolves the same VM name and expanded commands without performing Git discovery

#### Scenario: Removed branch flag

- **WHEN** any execution command receives `--branch feat/x`
- **THEN** CLI parsing fails rather than interpreting it as an identifier

### Requirement: Expanded names reject slashes

Spry MUST reject an effective VM name containing `/` before VM operations. The error MUST suggest a slash-free name rather than a removed Git placeholder.

#### Scenario: Slash in explicit name

- **WHEN** `--sprite app/task` is supplied
- **THEN** Spry fails before listing or creating, with no suggestion to use `{{branch_slug}}`

## REMOVED Requirements

### Requirement: Missing placeholder fails that value

**Reason**: Git-specific missing-value scenarios are replaced by explicit identifier and execution-boundary behavior.

**Migration**: Supply identifier only where the effective name or selected command references it; use the supported placeholders and retain lazy command failure semantics.

### Requirement: Placeholders expand in teardown commands

**Reason**: The former Git-inclusive teardown expansion contract is replaced by identifier/sprite/org-only expansion.

**Migration**: Replace Git placeholders in cleanup scripts. Missing or removed values still stop that cleanup step and prevent destruction.

### Requirement: Git discovery from cwd

**Reason**: Environment identity is explicit and independent of Git metadata.

**Migration**: Pass `--identifier` and use `{{identifier}}` for environment labels. Use literal repository/branch values or perform Git operations in recipe scripts; `--identifier` does not perform a checkout.

### Requirement: Branch slug for sprite names

**Reason**: Git-derived labels and implicit slugification are removed.

**Migration**: Supply an already-valid identifier or a literal VM name. Spry validates rather than transforming identifiers.
