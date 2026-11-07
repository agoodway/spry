# Design

## Context

See proposal.md for motivation and scope. Config currently deserializes flat YAML into recipe/override structures; setup, app, and teardown resolve Git metadata before passing commands through the shared step runner. All phases already share ordered VM/host execution. The change should replace config and template inputs without replacing that execution architecture.

## Goals / Non-Goals

**Goals:** Keep environment selection deterministic across commands; separate lifecycle and app configuration; preserve the existing execution and failure boundaries; make migrations fail visibly rather than selecting an unintended VM.

**Non-Goals:** Resume/wake hooks, remembered environments, automatic identifiers, setup completion tracking, automatic Git synchronization, new service supervision, new environment variables, a migration command, or broader runner refactoring.

## Decisions

### 1. Structured TOML, normalized into the existing execution model

Use the maintained `toml` crate with TOML 1.0 support, including heterogeneous string/table arrays, and strict root/table/host-step fields. Select its current compatible release during implementation and lock it in `Cargo.lock`; do not use pre-TOML-1.0 parsers such as `toml` 0.5. The complete public schema is:

```toml
[sprite]
name = "myapp-{{identifier}}"
org = "myteam"

[lifecycle]
setup = [
  "./scripts/setup",
  { host = "./scripts/register '{{sprite}}'" },
]
teardown = [{ host = "./scripts/cleanup '{{sprite}}'" }]

[app]
start = ["sprite-env services start app"]
stop = ["sprite-env services stop app"]
```

Tables, identity fields, and command lists are optional at parse time. Commands still require a resolved VM name; empty name/org values remain absent. Plain strings execute in the VM and `{ host = "..." }` tables execute on the host, in array order. TOML multiline literal strings are supported through normal TOML syntax. Keep the normalized recipe lists so runners need not know the external table layout. Do not introduce a task graph or per-step execution options. Reject unknown fields rather than preserving YAML's permissiveness: ignoring `resume` or a misspelled phase would hide lost behavior.

Use a dedicated strict host-step shape and preserve actionable diagnostics through any untagged step enum: invalid steps must identify the config file, phase/index, and expected string or `{ host = "..." }` shape, including the offending field/type. A generic "did not match any variant" alone is insufficient. Legacy root fields such as `name` or `setup` should point to `[sprite]` or `[lifecycle]` rather than merely reporting an unknown field.

### 2. Explicit identifier with no Git fallback

Expose the long flag `--identifier` on setup, app start, app stop, and teardown; do not add a short alias or config default. A supplied identifier must match `[A-Za-z0-9][A-Za-z0-9_-]*`, with case preserved and no trimming, slugification, or generated value. The alphanumeric first character avoids leading-option ambiguity; dots remain disallowed, including `..` in host paths. Validate supplied values before VM operations, even if unused. A missing identifier fails only when expansion encounters `{{identifier}}` in the effective name or selected command list; unused phases must not require it. `--sprite` still overrides the configured name before expansion and is independent of `--identifier`.

Names support only `{{identifier}}`. Commands support `{{identifier}}`, `{{sprite}}`, and `{{org}}`. Org itself is not templated. Keep exact, case-sensitive, nonrecursive raw substitution and existing name slash rejection, but replace the branch-slug error hint. There is no automatic identifier environment variable; host commands keep their existing `SPRITE` and optional `ORG` variables.

Remove GitInfo, discovery, branch overrides, slugification, and Git-only tests/module exports when no consumers remain. Change the template source of truth directly rather than retaining an adapter with dummy Git metadata. Recipe scripts may invoke Git themselves. Verbose output reports identifier (value or absent), not Git context.

### 3. Preserve phase boundaries

Setup creates only when needed and runs its full list on each invocation, including existing VMs. Setup does not implicitly run app start. App start/stop require an existing VM and run only their selected list. Teardown requires an existing VM, runs cleanup in order, and destroys only after success unless `--no-destroy` is set. A failed cleanup prevents destroy. Empty lists keep existing semantics. No resume CLI, config key, or automatic wake behavior is introduced.

Keep lazy command expansion: name errors occur before listing/provisioning; command errors occur immediately before that command, with prior commands possibly completed. Teardown expansion errors prevent destroy. Dry-run uses the same expansion and validation path, permits list, and performs no mutation. Do not change this to global template preflight in this change.

### 4. Explicit TOML cutover

At each directory, search `.spry.toml` then `spry.toml`. If neither exists but `.spry.yaml` or `spry.yaml` exists, stop with an actionable migration error instead of walking past it to an ancestor recipe. In the same directory, TOML wins over legacy YAML. Explicit config bypasses discovery; explicit `.yaml`/`.yml` paths fail with migration guidance, and other paths are parsed as TOML. No YAML parser or dual-format execution remains.

When a selected TOML file has `.spry.yaml` or `spry.yaml` in its directory, warn on stderr before VM operations, naming the selected TOML and every ignored legacy recipe. Apply this to explicit configs and dry-run too. Warn once per invocation, not per step, without parsing YAML; no warning is needed for a legacy file in an unselected directory. TOML remains authoritative and execution is not blocked.

Init writes `.spry.toml` with the directory basename as a literal `sprite.name`, empty `lifecycle.setup`, and commented identifier/app/teardown examples. Keep a runnable fixed-name default rather than requiring an identifier immediately. Custom paths and `--force` retain their behavior, but reject `.yaml`/`.yml` output paths to avoid generating a recipe the loader refuses. Existing YAML is never overwritten or converted automatically. After a successful write beside `.spry.yaml` or `spry.yaml`, print a migration note on stderr naming the files and warning that discovery will prefer TOML; ask the operator to verify the VM name and cleanup steps before running commands. Apply this to custom output directories and force writes, but do not print a success note on a failed write.

### 5. Focus verification on public contracts

Adapt the existing parser, CLI, and runner tests rather than retaining Git fixtures. Use distinct setup/start/stop/teardown commands to detect phase mixing, mixed VM/host arrays to detect reordering, and differing `--sprite` and identifier values to detect conflation. Check missing/invalid identifiers, removed placeholders/flags, strict schema errors, child legacy configs versus ancestor TOML, and dry-run/no-destroy/failure behavior. Parse generated init output through the real loader, including escaping-sensitive directory names.

The delta specs explicitly remove obsolete YAML/Git-specific requirements and add their named replacements where scenario titles must change. This records intentional contract replacement without retaining contradictory titles or bypassing OpenSpec's scenario-loss checks. Other requirements remain MODIFIED with their existing scenarios preserved.

## Risks / Trade-offs

- Breaking existing recipes → Provide README migration examples, file-specific errors, and explicit mapping from old fields to TOML tables. Preserve existing VM names during migration to avoid orphaning resources.
- No automatic Git-derived labels → Users pass an explicit identifier; scripts can choose labels outside Spry.
- Different identifiers can still resolve to one VM if a recipe omits them from its name → Document that the final resolved name, not the label, selects the VM.
- Raw command substitution is not shell escaping → Preserve and document the trusted-recipe boundary; keep identifier characters restricted and warn that org/VM values still need appropriate quoting.
- Lazy expansion can fail after provisioning or prior commands → Preserve current semantics and make errors phase/index-specific; recommend dry-run before mutation.
- Dual `app` and `stop` specs describe the same stop command → Update both contracts consistently without reorganizing the spec inventory.

## Migration Plan

1. Replace runtime YAML parsing with TOML and update the lockfile together with init/discovery changes.
2. Replace Git-based template inputs with explicit identifiers across all commands and remove dead Git-only code.
3. Update fixtures, tests, help, and README in the same change; run formatting, serial locked tests, Clippy, and a release build.
4. Document migration: move name/org into `[sprite]`, setup/teardown into `[lifecycle]`, start/stop into `[app]`; convert host maps to TOML inline tables; replace Git placeholders with explicit values or identifier-based scripts. Replace `--branch` with `--identifier` only when that label is intended to select the environment, not as a Git checkout operation. Existing dotted VM names such as `myapp-release-1.2` cannot be reconstructed with the restricted identifier: preserve them with a literal `sprite.name` or address them using `--sprite myapp-release-1.2`, including for teardown. Do not rewrite the name or create a replacement VM just to fit the identifier grammar.
5. At archive, refresh the config capability's Purpose from "YAML recipe" to "TOML recipe" and check the synchronized canonical specs for obsolete syntax. Purpose prose is not updated by a delta, so this is an explicit archive follow-up rather than an early change to the current contract.

Rollback is a binary/source rollback, with the original YAML retained by the user. Do not automatically edit recipes, rename VMs, remove resources, or migrate stored state.
