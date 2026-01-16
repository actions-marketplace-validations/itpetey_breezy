# Release Breezy

GitHub Action for continuous draft release generation.

## What it does

- Creates or updates a single draft release per branch (or per branch + directory).
- Uses merged PR titles as release notes.
- Resolves version numbers from language archetypes (e.g. Rust `Cargo.toml`).
- Optional `breezy.yml` config for grouping, templating, and tag/name formats.

### Supported languages/frameworks

- Rust
- NodeJS

Please raise an issue to request support for your language/framework of choice. PRs also very welcome :)

## Inputs

- `language` (optional): Language archetype(s) for version detection. If omitted, `breezy.yml` is used.
- `github-token` (required): GitHub token used to create/update releases.
- `tag-prefix` (optional): Prefix for tags when no `tag-template` is set. Default `v`.
- `config-file` (optional): Path to a `breezy.yml` config.
- `directory` (optional): Repo-relative directory containing the manifest to read (scopes drafts per branch + directory).

Use `directory` when your repo has multiple sub-projects/manifests and you want independent draft releases per sub-project on the same branch.

## Config file (`breezy.yml`)

By default, Breezy looks for `.github/breezy.yml` in the repo, or `$HOME/.github/breezy.yml` inside the container. You can also pass `config-file` explicitly.

Example:

```yml
language: rust
tag-template: $DIRECTORY-$VERSION
name-template: $DIRECTORY-$VERSION
categories:
  - title: Features
    labels:
      - feature
      - enhancement
  - title: Bug Fixes
    labels:
      - fix
      - bugfix
      - bug
  - title: Maintenance
    label: chore
exclude-labels:
  - skip-log
change-template: "* $TITLE @$AUTHOR ($NUMBER)"
template: |
  # Changes

  $CHANGES
```

Category headings can be set with `title` (defaults to `h2`) or with `h1`, `h2`, or `h3` keys to control the heading level. Use only one of these keys per category.

Example heading levels:

```yml
categories:
  - h1: Breaking Changes
    label: breaking
  - h2: Features
    labels: [feature, enhancement]
  - h3: Maintenance
    label: chore
```

Template variables:

- `$VERSION`: Resolved version.
- `$DIRECTORY`: Directory input (empty when not set).
- `$TITLE`: PR title.
- `$AUTHOR`: PR author login.
- `$NUMBER`: PR URL.
- `$CHANGES`: Rendered change list (only for the top-level `template`).

## Example workflow

```yml
name: Release Breezy
on:
  push:
    branches: [main]

jobs:
  draft:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: read
    steps:
      - uses: actions/checkout@v4
      - uses: ./
        with:
          language: rust
```

## Example directory workflow

```yml
name: Release Breezy
on:
  push:
    branches: [main]

jobs:
  draft:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: read
    strategy:
      matrix:
        directory:
          - crates/app
          - crates/worker
    steps:
      - uses: actions/checkout@v4
      - uses: ./
        with:
          language: rust
          directory: ${{ matrix.directory }}
```

## Prior art

This action is heavily inspired by [release-drafter](https://github.com/release-drafter/release-drafter). There are a few key differences:
- `breezy` does not attempt to increment the version number - it reads directly from the appropriate manifest
- `breezy` creates a single release draft per branch by default
- `breezy` supports multiple sub-projects with separate releases for each
