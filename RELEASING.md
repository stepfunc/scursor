# Releasing

A release is driven by **pushing a git tag**. When a tag `X.Y.Z` is pushed, CI
(`.github/workflows/ci.yml`) runs every check, then publishes that version to
crates.io and opens a draft GitHub release. Your job is to land a release commit
on `main`, then tag it.

## 1. Prepare a `Release X.Y.Z` PR against `main`

- Set `version` in `Cargo.toml` to `X.Y.Z`.
- Make sure the top of `CHANGELOG.md` is a `### X.Y.Z ###` section describing the
  user-facing changes since the last release (see the conventions below). Its
  contents become the GitHub release notes.
- Decide whether the release is a patch or minor version. This crate is `0.x`, so
  a breaking change bumps the minor version (`0.5.1` → `0.6.0`) and anything else
  bumps the patch version.

Open the PR, get CI green, and merge it. CI runs `cargo publish --dry-run` on
every PR, so packaging problems show up here rather than at release time.

## 2. Tag the release on `main`

The tag must be the bare version (no `v` prefix), and must match `Cargo.toml`
exactly:

```bash
git checkout main && git pull
git tag -a X.Y.Z -m "Release X.Y.Z"
git push origin X.Y.Z
```

**Publishing to crates.io is irreversible.** A version can be yanked but never
replaced, so check the version before pushing the tag.

Before publishing, CI fails the release if the tag doesn't match the
`Cargo.toml` version, if `CHANGELOG.md` has no `### X.Y.Z ###` section, or if the
tagged commit isn't on `main`.

## 3. Finish up

- Watch the tag's CI run. Each release step skips work that's already done, so
  if something fails partway, rerunning the workflow is safe.
- Review the **draft** GitHub release CI created, then publish it.

## CHANGELOG conventions

- One `### X.Y.Z ###` section per release, newest first.
- Bullet prefixes: `:star:` feature, `:bug:` bug fix, `:book:` documentation,
  `:wrench:` tooling and CI.
- Reference the issue or PR, e.g. `See [#16](https://github.com/stepfunc/scursor/issues/16).`

## One-time setup: crates.io trusted publishing

CI publishes with [trusted publishing](https://crates.io/docs/trusted-publishing),
so no API token is stored in GitHub. A crate owner configures it once on
crates.io, under the crate's settings, with:

- Repository owner: `stepfunc`
- Repository name: `scursor`
- Workflow filename: `ci.yml`
- Environment: leave blank
