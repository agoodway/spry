## 1. Template expansion

- [x] 1.1 Add `template` module: parse `{{name}}`, expand known placeholders, reject unknown / interior whitespace, do not re-expand values
- [x] 1.2 Implement `branch_slug` (non `[A-Za-z0-9._-]` runs → `-`, strip edges, preserve case)
- [x] 1.3 Tests for placeholder set, unknown token, slug of slashed branch, collapse, empty slug, no re-expansion

## 2. Git context

- [x] 2.1 Add `GitInfo` (`branch`, `commit`, `remote`) and `git_info(cwd)` via `git -C` (`rev-parse --abbrev-ref HEAD`, `rev-parse HEAD`, `remote get-url origin`); missing git or failed command → that field absent
- [x] 2.2 Treat `HEAD` from abbrev-ref as absent branch (detached)
- [x] 2.3 Apply `--branch` onto `GitInfo` for branch/slug only
- [x] 2.4 Tests with a temp git repo (and a non-repo dir)

## 3. Setup wiring

- [x] 3.1 Expand resolved name (recipe or `--sprite`) before list/create; fail if expanded name contains `/`; fail missing placeholders before list
- [x] 3.2 Expand each setup command immediately before exec / dry-run print; fail that command if a placeholder is missing
- [x] 3.3 Add `--branch` on `spry setup`; GitInfo test seam so setup tests do not need git
- [x] 3.4 `--verbose` prints branch, branch_slug, commit, remote (value or `absent`)
- [x] 3.5 Tests: unique name per branch, `--sprite` expansion, `--branch` without git, dry-run expanded checkout, name missing git, setup line missing remote after first command, no placeholders without git

## 4. Init and docs

- [x] 4.1 Comment `{{branch_slug}}` / `{{remote}}` / `{{branch}}` examples in the starter recipe; keep basename name and empty setup; still no git requirement
- [x] 4.2 README: placeholders, `--branch`, example `name: myapp-{{branch_slug}}` plus clone/checkout lines
