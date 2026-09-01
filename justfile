# Project automation recipes

# Start the CLI
up:
    cargo run

# Deploy to production
deploy:
    @echo "TODO: Configure deployment"

# Run tests
test:
    cargo test --locked

# Run linting
check:
    cargo clippy --locked --all-targets --all-features -- -D warnings

# Build the project
build:
    cargo build --locked --release

# Clean build artifacts
clean:
    cargo clean

# Tag the Cargo.toml version and trigger the GitHub release workflow
release:
    #!/usr/bin/env bash
    set -euo pipefail

    git remote get-url origin >/dev/null

    if [[ -n "$(git status --porcelain)" ]]; then
      echo "error: commit or stash all changes before releasing" >&2
      exit 1
    fi

    package_version="$(cargo pkgid | sed -E 's/.*[#@]//')"
    tag="v${package_version}"

    if git rev-parse --verify --quiet "refs/tags/${tag}" >/dev/null; then
      echo "error: tag ${tag} already exists" >&2
      exit 1
    fi

    cargo test --locked
    git tag --annotate "$tag" --message "Release ${tag}"
    git push origin "$tag"
