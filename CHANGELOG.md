# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.1 — 2026-11-15

### Changed

- Release binaries are Linux-only (x86_64-unknown-linux-gnu,
  aarch64-unknown-linux-gnu). The v0.1.0 workflow built macOS archives as
  well; those are dropped in favor of a reliable, fast pipeline.

## v0.1.0 — 2026-11-15

First public release.

### Added

- Accounts and sessions: register, login, logout (revokes the server-side
  session and clears the local keyring entries), short-lived access tokens
  with a rotating refresh token stored in the OS keyring. Slots are derived
  per server (`<token_account>-<fnv1a64(base_url)>`) or namespaced explicitly
  with `--profile <name>`.
- Project lifecycle: `init` (fail-fast pre-flight before any remote call,
  idempotent repair on re-run, `--adopt` for existing projects), `add`,
  `delete project`, `delete environment`.
- Variable sync: `push` (create, conflict prompts, never deletes remote-only
  variables) and `pull` (preserves comments and formatting). Values are hidden
  by default (`--show-values` to reveal); `--dry-run` supported. `pull
  --project-id` works for shared projects without a local binding.
- Sharing: `invite`, `invites list/accept/cancel`. Accepting an invitation
  links the current directory (writes `.sharenv/config.toml` with the project
  id and identity metadata; never overwrites an existing binding).
- Member administration (owner-only, re-authenticates): `members role`,
  `members environment`.
- Machine-readable output: `--json` for `list projects|environments|members`,
  `whoami`, `push` and `pull`; human output goes to stderr in JSON mode.
- Releases: prebuilt binaries for Linux (x86_64, aarch64) and macOS
  (x86_64, aarch64) attached to the GitHub release, checksums in `SHA256SUMS`;
  `cargo install sharenv` from crates.io. MIT licensed.

### Security

- Local secret handling via `fs_secure`: `0600` files, `0700` directories,
  atomic writes, symlink/permission/ownership checks with actionable errors.
- Non-loopback plaintext `http://` is refused by default; `--insecure` is a
  per-invocation escape hatch that cannot be persisted.
- Pushing to a project you do not own always asks for confirmation; pulling
  from a non-member is refused before any variable is fetched.
