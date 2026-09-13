# Inspector Review-Update — add-spry-cli

**Reviewed:** 2026-08-31
**Reviewer:** inspector-review-update
**Mode:** auto
**Verdict:** Ready to implement

## Summary

This change defines a greenfield Rust CLI that initializes a repo-local Sprite recipe and provisions/configures a VM through the `sprite` CLI. The review found one blocking invocation issue plus five specification and coverage gaps; all were patched. The resulting artifacts are internally consistent and pass strict OpenSpec validation, with application-code alignment necessarily limited because the workspace intentionally contains no implementation yet.

**Original counts:** Critical: 1 · Warning: 3 · Suggestion: 2
**Patches:** Auto-patched: 5 · User-guided: 0 · Model-recommended: 1 · Skipped: 0

## Scope inspected

- Proposal: `openspec/changes/add-spry-cli/proposal.md`
- Design: `openspec/changes/add-spry-cli/design.md`
- Tasks: `openspec/changes/add-spry-cli/tasks.md`
- Deltas:
  - `openspec/changes/add-spry-cli/specs/config/spec.md`
  - `openspec/changes/add-spry-cli/specs/init/spec.md`
  - `openspec/changes/add-spry-cli/specs/setup/spec.md`
- Canonical specs consulted: none; `openspec/specs/` is absent in this greenfield workspace
- Other active changes consulted: none

## Patches applied

5 findings were auto-patched. 0 were patched after user guidance. 1 was patched from a model recommendation. 0 were skipped.

### Auto-patched

1. **Make Sprite creation non-interactive** — `openspec/changes/add-spry-cli/design.md:43` → Added `--skip-console` to the pinned create invocation and aligned the setup requirement, implementation task, and argv/test coverage. The current Sprite CLI documents this flag as the way to exit after creation instead of connecting to a console.
2. **Specify the success summary** — `openspec/changes/add-spry-cli/specs/setup/spec.md:119` → Added a normative requirement and scenario covering VM name, created/existed state, command count, and elapsed time; aligned task 5.6 and test coverage.
3. **Specify verbose diagnostics** — `openspec/changes/add-spry-cli/specs/setup/spec.md:128` → Added a requirement and scenario for resolved config/name/org and complete Sprite command lines.
4. **Require progress labels on every setup step** — `openspec/changes/add-spry-cli/specs/setup/spec.md:102` → Required `[i/n]` labels for all invoked setup commands, added scenario coverage, and aligned task 5.5.
5. **Cover init outside Git** — `openspec/changes/add-spry-cli/specs/init/spec.md:19` → Added an explicit no-Git scenario and aligned task 4.4 test coverage.

### Model-recommended patches

1. **Define the generated starter name** — `openspec/changes/add-spry-cli/specs/init/spec.md:5` → Required `name` to come from the current directory basename and required `setup: []`; aligned the design, tasks, and default-init scenario.
   - **Chose:** Derive `name` from the current directory basename and emit an empty setup list.
   - **Rationale:** This matches the repo-local intent and makes the generated recipe immediately useful without adding prompting or broader naming behavior. It is the smallest deterministic choice supported by the proposal.

### Skipped

_None._

## Critical

_None._

## Warning

_None._

## Suggestion

_None._

## Alignment notes

- **Other active changes:** No other unarchived changes exist, so there are no overlapping deltas or conflicts.
- **Canonical specs:** No canonical capability specs exist yet; all three capabilities are correctly declared as ADDED for this greenfield CLI.
- **Codebase assumptions verified:** The workspace contains no application source, Cargo project, or Git repository, matching the proposal's greenfield premise. The external `sprite create --skip-console` behavior was verified against the official Sprite CLI command reference. Other implementation-level paths and symbols cannot be checked until source exists.

## What looks good

- The proposal, design, tasks, and three delta specs agree on a deliberately small two-command product.
- Config discovery and precedence are explicit, repo-only, and independent of Git.
- Setup behavior covers list/create failures, no-create, dry-run, fail-fast execution, empty recipes, and optional organization scoping.
- The injected client and fake-client test strategy cleanly separates orchestration tests from live Sprite infrastructure.
- `openspec validate add-spry-cli --strict` passes after the patches.
