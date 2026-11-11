# sharenv security model

This document describes what the `sharenv` CLI protects, what it does not
protect, and the assumptions behind those claims.

## Summary

`sharenv` is a **plaintext-to-server** client. It is **not end-to-end
encrypted**.

- `.env.*` files are read locally as plaintext.
- Values are sent to the `sharenv-api` server as JSON over the network.
- The server decrypts stored values before returning them and during normal
  operation, so the server operator can read every value.
- Authentication tokens are stored in the operating system keyring.
- Local `.env.*` files and the token's access to the server are the only
  secrets the CLI itself holds.

The CLI can reduce accidental exposure, but it cannot protect secrets from a
compromised server, a stolen account, or a compromised machine.

## Data flow

```text
.env.dev  --plaintext-->  sharenv CLI  --TLS + bearer token-->  sharenv-api
                                                                      |
                                                          decrypts / encrypts
                                                                      |
                                                                   database
```

When pulling:

```text
database  -->  sharenv-api (decrypts)  --TLS-->  sharenv CLI  -->  .env.dev (plaintext)
```

There is no key material on the client and no client-side encryption of
variable values.

## Connections and certificate requirements

- TLS is rustls with the platform trust store (`rustls-platform-verifier`) as
  the default — public CAs work out of the box.
- Self-hosted deployments with a private CA: pass `--cacert <path>` on the
  invocation (PEM bundle). It is **additive** to the platform roots, never
  persisted in config or environment variables.
- The default local deployment (`http://localhost:8000`) is explicitly allowed;
  any other plaintext `http://` host is refused unless `--insecure` is passed
  for that single invocation.
- The bearer token is never forwardable via redirects: the client does not
  follow them.
- Read-only requests are retried up to 3 times with 250/750ms backoff on
  connection errors, timeouts, 5xx and 429. Writes are never retried.

## What `sharenv` protects against

- **Loose permissions on local secret files**: newly created files are
  `0600`, `.sharenv/` is `0700`, and the CLI refuses to read or write files
  that are symlinks, not owned by you, or have group/other bits set.
- **Accidental commits of `.env*`** when the repository's `.gitignore` covers
  them. `init` does not yet modify `.gitignore`, so add `.env*` yourself until
  that hardening lands (see the project plan).
- **Casual terminal exposure** when pushing or pulling: values are hidden by
  default and only printed with an explicit `--show-values`.
- **Terminal escape injection** from remote values or error bodies: printed
  strings are stripped of ANSI escapes and control characters are shown
  escaped.
- **Plaintext HTTP to a remote host** by default: non-loopback `http://` URLs
  are rejected unless `--insecure` is passed for that single invocation.
- **Hanging or misbehaving servers**: connect timeouts (10s), request timeouts
  (30s), a bounded connection pool, no redirect following (your bearer token can
  never be forwarded by the server), and idempotent-GET retries with backoff.
- **Silent transfers to an unexpected project**: the project name and owner are
  printed before pushing or pulling, and pushes to a project you do not own
  require confirmation.
- **Pulling projects you do not belong to**: `pull` resolves the project from
  the server's project list first. Non-members are refused before any variable
  is fetched.
- **Plaintext token storage on disk**: tokens live in the OS keyring rather
  than in a config file. Slots are derived per server, so dev and production
  sessions never overwrite each other.
- **Loose local file permissions**: `.env.*` and `.sharenv/` are created and
  enforced `0600`/`0700`; all writes go through a temp file + atomic rename.
  Symlinks and group/other-permission files are refused outright.

## What `sharenv` does not protect against

- **A compromised or malicious server operator.** Server-side encryption is not
  end-to-end encryption. The API can decrypt every value.
- **A stolen bearer token.** A token grants the same access as the user until
  it expires (~15 minutes). `logout` revokes the token server-side and wipes
  the local keyring entries.
- **A compromised local account.** Any process running as your user (or root)
  can read `.env.*`, the keyring, and memory while the CLI is running.
- **Local copies outside the CLI's control.** Editors, shell history, swap
  files, crash dumps, backups, snapshots, and copy-on-write/SSD remnants may
  retain plaintext secrets after the file is deleted.
- **A malicious repository.** A repository can contain a hand-written
  `.sharenv/config.toml`. The CLI treats that file as advisory and shows the
  project identity from the server, but it cannot make a hostile repository
  safe. Review `.sharenv/config.toml` before pushing.
- **An unwanted share, once accepted.** Sharing uses invitations and the
  invited account must accept them. An owner who invites you can grant access
  to selected environments; re-check your memberships with `list members` and
  revoke access there (or ask the project owner).
- **Server logs and backups.** The server may retain request metadata, and
  database backups may contain encrypted values. Refer to the API's security
  plan.

## Transport

The API base URL is taken from the global `config.toml` or the
`SHARENV__API_CONFIG__BASE_URL` environment override.

- `https://` is required for any non-local host.
- `http://` is accepted only when the parsed host is `localhost`, an IPv4
  loopback address, or IPv6 `::1`.
- Any other plaintext URL fails before credentials are sent, with an error that
  names the host.
- `--insecure` allows plaintext HTTP for one invocation and prints a warning.
  It is intentionally **not** available as a config key or environment
  variable, so it cannot become a permanent downgrade.

TLS certificates are verified by the platform trust store. Self-hosted
deployments should use a certificate trusted by the operating system.

## Authentication and tokens

- `login`/`register` obtain a bearer JWT and store it in the OS keyring using
  the `[auth]` service name in `config.toml`. The account is
  `<token_account>-<fnv1a64(base_url) hex>` per server, or
  `<token_account>-<profile>` with `--profile <name>`.
- All project, environment, and variable requests attach the token.
- `--profile <name>` (per invocation, never persisted) namespaces the slot
  for that run; dev and production servers never overwrite each other.
- If the keyring backend is unavailable (for example, a headless environment
  without a secret service), token storage fails. The CLI does not fall back to
  a plaintext token file.


## Secret display

`push` and `pull` hide values by default:

```text
DATABASE_URL changed
```

`--show-values` prints values after sanitizing terminal control characters and
truncating long values. Values are still only shown for that invocation and are
never written to logs by the CLI itself.

For machine-readable output, `--json` (per-invocation global flag) emits a
single JSON document on stdout for `list projects|environments|members`,
`whoami`, and the `push`/`pull` summaries; progress messages go to stderr.
Values only appear with `--show-values`.

Do not pass secrets as command-line arguments; they would be visible in shell
history and process listings.

## Project identity and local config

At `init`, the CLI records the project id, name, owner id, API base URL, and a
`created_by = "init"` marker in `.sharenv/config.toml`.

Before `push` or `pull`, the CLI fetches the project list from the server and
prints:

```text
Project: my-project (owner: you)
```

The local identity fields are **advisory**, not a security boundary: anyone who
can write the file can forge them. They are used to detect drift:

- A name or owner mismatch blocks `push` and requires confirmation for `pull`.
- A missing identity marker (for example, a config written by an older version
  or by hand) produces a warning and requires confirmation before pushing.
- A changed API base URL produces a warning.

Pushes to a project you do not own always show the owner and ask for
confirmation, even when the local config looks valid. There is no `--yes`
bypass for that prompt.

`pull --project-id <project_id>` resolves the project name and owner before fetching
variables, refuses projects you are not a member of, and asks for confirmation
when you are a member but not the owner.

Run `init --adopt` to record identity metadata for an existing, already
initialized project without creating a new one.

## Recommendations

- Use HTTPS with a certificate from a trusted CA.
- Run `init --adopt` after upgrading if the CLI warns about missing identity
  metadata, then verify the printed project name and owner.
- Keep `.env*` out of version control.
- Review `.sharenv/config.toml` when working in a cloned or untrusted
  repository.
- Do not share tokens or paste them into issue reports.
- Prefer `--show-values` only on a private terminal, and never in CI logs.
- Rotate credentials and secrets if a token or server is suspected to be
  compromised.

## Reporting a vulnerability

See `SECURITY.md` in the repository for the disclosure contact and supported
versions. Do not include real secrets in a report.

## Local file permissions

- `.env.<env>` files and `.sharenv/config.toml` are created with mode `0600`
  (owner read/write only). `.sharenv/` itself is `0700`.
- Writes replace the target atomically (temp file + `fsync` + `rename`), never
  writing through a symlink; a symlink at the target path is refused outright.
- Before any read or write of a secret file, the CLI checks: regular file, not
  a symlink, owned by you, no group/other permission bits. Anything else is
  refused with an error naming the cause and the exact `chmod 600` fix.
- New files and any file the CLI rewrites are forced back to `0600`.
