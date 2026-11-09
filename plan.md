# sharenv CLI — security remediation plan

Companion plan for the API lives at `../sharenv-api/plan.md`.

This plan is ordered by risk. Work top-to-bottom where possible. Each task is a
single reviewable change. `P0` = do before using the CLI with real production
secrets, `P1` = important hardening, `P2` = polish.

Legend: `- [ ]` todo, `- [x]` done. Add the PR/commit next to a task when you
finish it.

---

## 1. Trust model and UX decisions

Decide what the CLI promises before changing behavior.

Settled decisions:

- Values are hidden by default; `--show-values` prints full sanitized values.
- Non-loopback `http://` is refused; `--insecure` is per-invocation only and
  cannot be persisted in config or environment variables.
- Push to a project the user does not own always confirms. Pull requires
  membership; non-members are refused before any variable is fetched.
- Local project identity is advisory. `init` records it and `init --adopt`
  records it for an existing project.

- [x] **P0 — Document the trust model.** Add `docs/security.md`: the CLI sends
      plaintext values to the API; the server decrypts; the OS keyring holds the
      token; local `.env.*` files are plaintext. State that a compromised server
      or stolen token exposes all values.
- [x] **P0 — Decide the default for secret display.** Never print values by
      default; print names and `(changed)` markers, with an explicit
      `--show-values` flag.
- [x] **P0 — Decide the default for non-HTTPS URLs.** Refuse non-HTTPS unless
      the host is loopback, with a per-invocation `--insecure` escape hatch.
- [x] **P1 — Define a "safe project" check.** Before push/pull, show the
      project name and owner, warn when the owner is not the current user or
      when local identity metadata is missing, and require membership for pull.

---

## 2. Credentials and token handling

Current code: `src/utils/token.rs`, `src/commands/login.rs`,
`src/commands/register.rs`, `src/utils/prompt.rs`, `src/settings/models.rs`.

- [ ] **P0 — Add a `logout` command.** Call the new server logout/revocation
      endpoint (see server plan §3) and delete the keyring entry. Wire it into
      `src/cli.rs`, `src/commands/mod.rs`, and `src/commands/help.rs`.
- [ ] **P0 — Stop leaving tokens valid after logout.** Ensure `logout` clears
      the local token even if the server call fails, and warns the user that the
      token may still be valid until expiry.
- [ ] **P1 — Zeroize token/password buffers.** Use `zeroize`/`secrecy` for the
      token, password, and decrypted values held in memory.
- [ ] **P1 — Support named profiles/accounts.** The fixed
      `token_service`/`token_account` means one token per OS user
      (`config.toml:16-18`). Add `--profile` or per-base-URL keyring accounts so
      dev/prod tokens do not overwrite each other.
- [ ] **P1 — Reject insecure keyring fallbacks.** Confirm the keyring backend
      never falls back to a plaintext file; fail closed with a clear error in
      headless environments.
- [ ] **P1 — Add a token-expiry check.** Decode the JWT `exp` locally and prompt
      for re-login before a request that will fail with `401`.
- [ ] **P2 — Add a `whoami` command** that shows the logged-in user (from the
      JWT `sub`/server endpoint) without printing the token.

---

## 3. Transport and HTTP client hardening

Current code: `src/api/client.rs`, `src/api/*.rs`, `config.toml`,
`src/utils/error.rs`.

- [ ] **P0 — Enforce HTTPS by default.** In `ApiClient::new` or config
      validation, reject `http://` for non-loopback hosts unless `--insecure`
      is passed. Acceptance: `base_url = "http://prod.example.com"` fails with a
      clear error.
- [ ] **P0 — Add request timeouts.** `Client::new()` has none
      (`src/api/client.rs:15`). Set `connect_timeout` and `timeout` (e.g. 10s /
      30s), and a bounded pool.
- [ ] **P0 — Disable or constrain redirects.** Reqwest follows redirects by
      default. Set `redirect::Policy::none()` or a custom policy that refuses
      cross-host and HTTPS→HTTP redirects, so the bearer token cannot be
      forwarded.
- [ ] **P0 — Sanitize server-provided error text.** `api_error` echoes the raw
      body (`src/utils/error.rs:31-57`). Strip ANSI/control characters and
      truncate before printing. Never print a body that could contain a secret.
- [ ] **P1 — Pin or document TLS trust.** Use the platform verifier (already the
      default via rustls), add a `--cacert` option for self-hosted deployments,
      and document certificate requirements.
- [ ] **P1 — Add a `User-Agent`** with the CLI version for server audit logs.
- [ ] **P1 — Add retry/backoff only for idempotent GETs**, never for writes, to
      avoid duplicate pushes.

---

## 4. Local file security

Current code: `src/utils/env_file.rs`, `src/utils/local_config.rs`,
`src/utils/env_parser.rs`.

- [ ] **P0 — Create `.env.*` files with `0600`.** `File::create`/`fs::write` use
      the umask; on this machine `.env.dev` is `-rw-r--r--`. Use
      `OpenOptions::new().mode(0o600)` (Unix) and set `0600` on existing files
      after write. Acceptance: `ls -l .env.dev` shows `-rw-------`.
- [ ] **P0 — Create `.sharenv/` with `0700` and `config.toml` with `0600`.**
      `local_config::save` currently uses `fs::create_dir_all` + `fs::write`
      (`src/utils/local_config.rs:19-30`).
- [ ] **P0 — Refuse to read/write unsafe files.** Before reading or writing a
      secret file, check: it is a regular file, not a symlink, owned by the
      current user, and has no group/other permissions. Offer a `--fix-perms`
      or clear error.
- [ ] **P0 — Make writes symlink-safe.** Use `O_NOFOLLOW`/`create_new` or
      `lstat` checks. A malicious repo with `.env.dev -> ~/.bashrc` or
      `.sharenv/config.toml -> ~/.ssh/authorized_keys` can currently cause
      arbitrary overwrites (`env_file.rs:20-34,38-76`,
      `local_config.rs:19-30`).
- [ ] **P1 — Make writes atomic.** Write to a temp file in the same directory,
      `fsync`, then `rename`. Prevents truncation/corruption on crash and
      reduces TOCTOU windows.
- [ ] **P1 — Validate the target path stays inside the project root.**
      Canonicalize the parent and reject paths that escape the cwd.
- [ ] **P1 — Respect a safe umask** and do not silently widen permissions on
      existing files.

---

## 5. Secret output and terminal safety

Current code: `src/commands/push.rs`, `src/commands/pull.rs`,
`src/utils/env_changes.rs`.

- [x] **P0 — Stop printing secret values by default.** `push` prints
      `remote:`/`local:` values (`push.rs:139-141`); `pull` prints every new and
      changed value (`pull.rs:140-142,154-156`). Print only names and change
      type. Add `--show-values` for explicit reveal.
- [x] **P0 — Sanitize all remote/local strings before printing.** Strip ANSI
      escapes, `\r`, and other control characters to prevent terminal escape
      injection. Apply to values, names, error bodies, and project/env names.
- [x] ~~**P1 — Mask values in `--show-values` mode** by default (e.g. first/last
      two characters) unless `--reveal` is also passed.~~ Superseded by the
      section 1 decision: `--show-values` prints full sanitized values.
- [ ] **P1 — Avoid writing secrets to shell history.** Do not accept secret
      values as command-line arguments; keep reading them from files/prompts.
- [ ] **P1 — Add a `--quiet`/`--json` mode** so CI can avoid values in logs and
      get machine-readable output.
- [ ] **P2 — Warn when stdout is not a TTY** that output may be captured in
      logs.

---

## 6. `.env` parsing and writing safety

Current code: `src/utils/env_parser.rs`, `src/utils/env_file.rs`,
`src/utils/env_changes.rs`.

- [ ] **P0 — Validate variable names on read and write.** Enforce a strict
      allowlist (same as the server; see server plan §5). This blocks names with
      newlines, `=`, `#`, `/`, spaces, and control characters.
- [ ] **P0 — Escape the name in `format_line`.** It currently writes
      `{name}={value}` with an unescaped name (`env_parser.rs:46-48`), so a
      malicious member can inject extra lines into every collaborator's `.env`
      on pull. After the allowlist, add a defensive check anyway.
- [ ] **P0 — Escape shell metacharacters in values.** `format_value` escapes
      `\`, `"`, newlines, and tabs but not `$` or backticks
      (`env_parser.rs:140-158`). If a user `source`s the `.env`, command
      substitution can execute. Escape `$` and `` ` `` (or write single-quoted
      values where possible).
- [ ] **P1 — Reject duplicate keys on write as well as read.** `read_env_file`
      rejects duplicates; ensure additions/updates cannot introduce one.
- [ ] **P1 — Preserve comments/formatting safely.** Keep the current in-place
      update behavior, but re-parse the result before writing and fail if the
      output is invalid.
- [ ] **P1 — Add a `--dry-run`** that shows the planned file changes without
      writing.
- [ ] **P2 — Support a configurable quote style** (`single`/`double`/`auto`) and
      document how values are escaped.
- [ ] **P2 — Add round-trip tests** for values containing `\n`, `\r`, `\t`,
      `"`, `'`, `\`, `$`, `` ` ``, `#`, spaces, and empty strings.

---

## 7. Project identity and supply-chain protections

Current code: `src/commands/init.rs`, `src/commands/push.rs`,
`src/commands/pull.rs`, `src/utils/local_config.rs`, `src/models.rs`.

This is the highest-impact CLI gap: a planted `.sharenv/config.toml` plus an
unconsented project membership can make `push` upload the victim's secrets to an
attacker's project.

- [x] **P0 — Show the project name and owner before push/pull.** Fetch the
      project metadata and print `Project: <name> (owner: <id/email>)` before
      any confirmation. Require an explicit `y` for pushes to a project the
      current user does not own.
- [x] **P0 — Store project name/owner in `.sharenv/config.toml`** at `init`
      time. On later commands, compare the stored identity with the server
      response and refuse to push if the owner changed; require confirmation
      when it is unknown.
- [ ] **P0 — Treat a pre-existing `.sharenv/config.toml` as untrusted.**
      `init` currently says "already initialized" for any project the user is a
      member of (`init.rs:35-50`), including an attacker's project. Add a
      warning and require `--adopt` to reuse an existing config.
- [x] **P0 — Require confirmation that names the destination project** for
      `push` and `pull --pid`. Do not rely on environment names alone.
- [ ] **P0 — Add a `status`/`inspect` command** that prints project ID, name,
      owner, role, environments, and local file paths without printing values.
- [ ] **P1 — Warn on `pull` when the remote project owner is not the current
      user**, and show a diff summary before writing.
- [ ] **P1 — Warn when `init` creates files in a git repo that does not ignore
      `.env*`** (see section 8).
- [x] **P1 — Add `--project-id` validation** so `pull --pid` cannot be pointed at
      an arbitrary ID silently; show the resolved project name first.
- [x] **P2 — Record the server `base_url` in the local config** and warn if it
      changes between pushes.

---

## 8. `.gitignore` and local lifecycle

Current code: `src/commands/init.rs`, `src/commands/add.rs`,
`src/commands/delete.rs`, `src/utils/env_file.rs`.

- [ ] **P0 — Add `.env*` to the target repo's `.gitignore` on `init`/`add`.**
      Create or append a clearly marked block, preserving existing content.
      Acceptance: a fresh `init` in a repo without `.gitignore` produces one
      that ignores `.env.dev`, `.env.local`, `.env.prod`, etc.
- [ ] **P0 — Warn (and require confirmation) if `.env.*` is already tracked by
      git.** Run `git ls-files --error-unmatch` and refuse to continue unless
      `--force`.
- [ ] **P1 — Add a `delete --local` or `clean` command** that removes local
      `.env.*` and `.sharenv/` after project deletion. The current `delete`
      leaves plaintext secrets on disk (`delete.rs`).
- [ ] **P1 — Offer secure deletion** (overwrite/truncate before unlink) for
      local secret files, and document that SSD/COW filesystems may retain
      copies.
- [ ] **P1 — Add a `doctor` command** that checks file permissions, symlinks,
      gitignore coverage, keyring availability, and HTTPS configuration.

---

## 9. Server-contract alignment

These depend on API changes in `../sharenv-api/plan.md`.

- [ ] **P0 — Use the shared variable-name regex** from server plan §5 and reject
      invalid names before sending.
- [ ] **P0 — Implement the invite/accept flow.** Replace `add -u <email>` with
      `invite <email>` and an `invites accept <token>` command once the server
      exposes it. Until then, print a warning that sharing is immediate and
      irreversible.
- [ ] **P0 — Implement `logout`** against the new revocation endpoint.
- [ ] **P0 — Handle `403 email_not_verified`** by prompting the user to verify
      and adding a `verify` command if the server supports it.
- [ ] **P0 — Handle role information.** Hide/deny destructive commands for
      `reader` members; show the current role in `status`.
- [ ] **P1 — Show project name/owner from `GET /project/`** once the server
      returns it.
- [ ] **P1 — Handle `409`/`404` consistently** for removed members and deleted
      projects, with actionable messages.

---

## 10. Error handling and user feedback

Current code: `src/utils/error.rs`, `src/commands/*`, `src/main.rs`.

- [ ] **P1 — Never print raw response bodies.** Use the sanitized message from
      section 3 and map common statuses to actionable text (`401` → login,
      `403` → no access, `404` → gone, `409` → already exists).
- [ ] **P1 — Do not leak secret values in error messages.** Ensure the variable
      name is printed but the value never is, even on update failures.
- [ ] **P1 — Add a `--verbose` mode** for debugging that still redacts values.
- [ ] **P2 — Exit codes:** distinguish auth failure, network failure, and
      validation failure for scripting.

---

## 11. Repository and release hygiene

- [ ] **P0 — Add secret scanning** (gitleaks/trufflehog) to CI/pre-commit.
- [ ] **P0 — Add dependency auditing** (`cargo audit`) and keep `reqwest`,
      `keyring`, and `tokio` current.
- [x] **P1 — Add a SECURITY.md** with a disclosure contact and supported
      versions.
- [ ] **P1 — Add a security section to the README** covering HTTPS, file
      permissions, `.gitignore`, and token storage.
- [ ] **P1 — Build with `--locked`** and verify `Cargo.lock` is committed (it
      is) so releases are reproducible.

---

## 12. Verification checklist

- [ ] **P0 — Permissions test:** newly created `.env.*` are `0600`; `.sharenv/`
      is `0700`.
- [ ] **P0 — Symlink test:** `.env.dev` and `.sharenv/config.toml` symlinks are
      refused, and the target is not modified.
- [ ] **P0 — Output test:** push/pull print no values by default;
      `--show-values` prints sanitized values only.
- [ ] **P0 — Injection test:** a remote variable named
      `FOO\nNODE_OPTIONS=--require /tmp/x.js` cannot be written to `.env`.
- [ ] **P0 — HTTPS test:** a non-loopback `http://` base URL is refused without
      `--insecure`.
- [ ] **P0 — Planted-config test:** a repo with `.sharenv/config.toml` pointing
      at a project the user is a member of but does not own shows the owner and
      requires explicit confirmation before `push`.
- [ ] **P0 — Gitignore test:** `init` in a repo without `.gitignore` adds
      `.env*`; already-tracked `.env.*` is detected.
- [ ] **P1 — Timeout/redirect test:** a hung server times out; a cross-host
      redirect does not receive the `Authorization` header.
- [ ] **P1 — Round-trip test:** values with shell metacharacters survive
      push/pull and do not execute when the file is sourced.
