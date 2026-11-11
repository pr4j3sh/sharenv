# sharenv — agent notes

Rust 2024 single-binary CLI that syncs project `.env` files with a remote sharenv-api. No workspace, no CI, no formatter/lint configs.

## Commands

```bash
cargo check --all-targets   # fastest signal
cargo build
cargo run -- <command> [flags]
```

- Aliases: `register`/`r`, `login`/`l`, `logout`, `whoami`, `verify`, `resend`, `init`/`i`, `add`/`a`, `invite`, `invites`, `members`, `list`/`ls`, `delete`/`d`, `help`/`h`/`--help`/`-h`. Command grammar: positional arguments are the primary entity; subcommands disambiguate (`delete project`, `members role`); flags are modifiers only.
- `members role <email> <writer|reader>` and `members environment <email>` — owner-only member administration (list members via `list members`). Role changes and grant changes re-authenticate with the owner's password. `members` depends on the API's `GET /project/{project_id}/members` endpoint; environments are resolved to names by the CLI.
- Grammar: primary entity is a positional argument; disambiguation is a subcommand; flags only carry modifiers. `init [name] [--adopt]`, `add <environment>`, `push|pull [environment] [--show-values]`, `pull [--project-id <id>]`, `delete project [name] | delete environment <name> [-y|--yes]`, `list projects|environments|members`, `members role <email> <writer|reader>` / `members environment <email>`. `--project-id=<id>` and space-separated both work. `--insecure` is a per-invocation global switch that allows non-loopback plaintext HTTP. There is no `-e`/`-p`/`-pid` anymore (hard break).
- Sessions: access tokens (~15 min) refresh transparently in `utils/session.rs` (`authenticated_client`), which every authenticated command uses; a failed refresh clears local tokens and asks for a new login. `register`/`verify` also save a `<slot>-pending-verify` keyring entry (the email) so `verify`/`resend` can find the account without re-entering it.
- Keyring slots are derived per server (`<token_account>-<fnv1a64(base_url) hex>`), or set explicitly per invocation with `--profile <name>` (never persisted in config or env). The refresh token lives at `<slot>-refresh`; pending verification email at `<slot>-pending-verify`. Zeroize boundary: passwords and token/refresh-token strings are zeroized after use; decrypted .env values are not (plaintext on disk anyway).
- On headless hosts the keyring (libsecret/Secret Service via zbus) fails closed with a clear error; no plaintext file fallback is ever used — do not add one.
- Project deletion (by id or name) requires the account password (server re-authentication); environment deletion does not. Invitation create/cancel likewise require the owner's password.
- Invitation acceptance moves membership server-side: local project state does not change until the invitee runs `invites accept <token>`.
- Run project commands (`init`, `push`, `pull`, `add`, `delete`, `list environments`) from the target project directory. This checkout may contain gitignored `.sharenv/config.toml` and `.env.*`; running from the repo root targets those.
- Smoke testing caveat: the OS keyring slot is per machine, not per `$HOME`. Never run smoke/e2e flows with the default `[auth]` account — it overwrites the real session. Set a throwaway account instead, e.g. `SHARENV__AUTH__TOKEN_ACCOUNT=smoke sharenv login`, and delete its keyring entries when done.

## Configuration: two different files

- **Global config**: `AppConfig::load` uses the OS config dir (`~/.config/sharenv/config.toml` on Linux) and auto-creates it from the root `config.toml` embedded via `include_str!` when missing. The root `config.toml` is compiled in, not read at runtime; editing it requires a rebuild.
- Overrides use `SHARENV` + `__`, e.g. `SHARENV__API_CONFIG__BASE_URL=...`. Array values (e.g. `environment_files.environments`) cannot be overridden by env vars.
- **Local project config**: `./.sharenv/config.toml` containing `[project] id = "..."`. Its directory/filename come from global `config_dir_name` / `config_file_path` and are resolved relative to the cwd. `.sharenv/` and `.env*` are gitignored.
- The global config is loaded from the OS config dir (`~/.config/sharenv/config.toml`), not the cwd — trust `src/settings/models.rs`, `src/settings/paths.rs`, and the root `config.toml`.
- Schema changes need a version bump in `src/settings/version.rs` plus a step in `src/settings/migrations.rs`; migrations back up to `config.toml.bak`.

## Architecture and conventions

- Entry flow: `src/main.rs` -> `cli::run_cli` -> `commands::*`. Dispatch is hand-rolled on `argv[1]` (no clap); add subcommands in both `src/cli.rs` and `src/commands/mod.rs`.
- Command flags are parsed by `src/utils/args.rs`; the dispatcher ignores extra args itself but passes them to the command.
- `src/utils/fs_secure.rs` is the single choke point for reading/writing secret files: checks (regular file, ownership, no group/other bits, no symlink) and atomic writes with `0600`/`0700`. Never call `std::fs` directly to touch `.env.*` or `.sharenv/*` — go through `fs_secure` (`safe_read`, `atomic_write_secure`, `ensure_secure_dir`, `check_secret_file`).
- `src/api/client.rs` builds a per-command `ApiClient`: connect timeout 10s, request timeout 30s, bounded idle pool, `redirect::Policy::none()`, `sharenv/<version>` User-Agent. Idempotent GETs are retried (250/750ms backoff, max 3 attempts) on connect/timeout/5xx/429 in `api/client.rs` (`send_get_with_retry`); writes are never retried. `--cacert <path>` (PEM, additive to platform roots via rustls-platform-verifier), `--profile <name>`, `--insecure` and `--json` are per-invocation global switches, never persisted in config or env. `--json` emits a single JSON document on stdout (list/whoami/push/pull); in JSON mode prompts and progress go to stderr so stdout stays parseable.
- Non-2xx responses become `ApiError` in `src/utils/error.rs`, surfacing the backend's `detail`; `409 Conflict` is used as "already exists" (push skips those).
- Tokens live in the OS keyring using `[auth]` service/account, not in a file: the access token at `<slot>`, the refresh token at `<slot>-refresh` (deleted by `logout`), and an optional pending verification email at `<slot>-pending-verify`, where `<slot>` is `token_account-<fnv1a64(base_url)>` by default or `token_account-<profile>` with `--profile`. Login/token reads can fail in headless environments without a secret service; they always fail closed.
- `_id` fields in `src/models.rs` intentionally match the backend JSON (`_id`); do not rename to `id`. Variable values accept JSON strings or other scalars.

## Behavior gotchas

- `init` uses the cwd basename unless a name is given as a positional argument, validates `[a-z0-9_-]+`, always creates a new remote project, saves its id and identity metadata locally, then creates every `environment_files.environments` entry remotely and as `.env.<env>` locally. If the local project id already matches a remote project, it reports “already initialized” and exits without creating anything; `init --adopt` records identity metadata for that existing project instead.
- `push <env>` creates missing variables and prompts to update existing ones whose remote value differs. It never deletes remote-only variables. Without an environment argument, it prompts and pushes every remote environment that has a local `.env.<env>` file. `pull <env>` prompts per difference and preserves existing comments/formatting; without an argument, it prompts and pulls every remote environment.
- `pull --project-id <project_id> <env>` works without a local `.sharenv` config (for shared projects); otherwise it needs the local project id. `pull` and `push` resolve the project from `GET /project/` first: non-members are refused, the project name/owner are printed, and transfers to projects the user does not own require confirmation.
- `invite <email>` shares the current project via an owner-created invitation (`POST /v1/project/{project_id}/invitations`, role + explicit environment grants + owner password re-authentication). Invitations are listed/canceled with `invites list` / `invites cancel <id>` and accepted by the invitee with `invites accept <token>` (`POST /v1/invitations/{token}/accept`). `add <environment>` creates that environment in the local project and an empty `.env.<env>` if missing. A `409 Conflict` (environment already exists) is reported as a no-op success.
- `delete project [name]` deletes the current project (by the id in `.sharenv/config.toml`) when no name is given, or by name through the API otherwise, and removes the local config only when the deleted id matches; `delete environment <env>` resolves the environment within the local project. Project deletion requires your password even with `-y` (server re-authenticates); environment deletion does not. All prompt unless `--yes`/`-y`; local `.env.*` files are never deleted.
- `list` requires a subject: `list projects`, `list environments` or `list members` (short aliases `p`/`e`/`m`; `members` is owner-only).
- Project/environment names must be lowercase ASCII letters, digits, `-`, or `_`. Env files are `<env_file_prefix>.<env>` (default `.env.dev`). The parser supports comments, single/double quotes, `export`, and inline comments, and rejects duplicate keys.

## Backend API

The sibling FastAPI repo `../sharenv-api` is the contract source of truth; authoritative routes are in `app/api/v1/*/routes.py`. Routes and the environment query param come from `[api_config.routes]`, not hardcoded paths. Current client calls:

- `POST /v1/auth/register` (returns `{success, verification_required}`), `POST /v1/auth/login` (returns `{token, refresh_token, email_verified}`; `403 email_not_verified` if unverified and a mail provider is configured)
- `POST /v1/auth/refresh` (single-use, rotated), `POST /v1/auth/logout` (revokes all tokens), `POST /v1/auth/verify` (`{email, otp}`), `POST /v1/auth/resend` (`{email}`; indistinguishable success for unknown/already-verified accounts)
- `GET|POST|DELETE /v1/project/` (lists include `role`, paginated), `DELETE /v1/project/{project_id}` and `DELETE /v1/project/name/{project_name}` (both take `{password}`), plus `POST /v1/project/{project_id}/invitations`, `GET|DELETE /v1/project/{project_id}/invitations[/{invitation_id}]`, `POST /v1/invitations/{token}/accept`, and owner-only `GET /v1/project/{project_id}/members` (returns owner + members with resolved emails and env grants; non-owners get a hidden 404) and `PATCH /v1/project/{project_id}/members/{member_id}` (`{environments, role?, password}`)
- `GET|POST /v1/environment/{project_id}`, `DELETE /v1/environment/{environment_id}` (GET accepts `?environment_name=`, paginated)
- `GET|POST /v1/variable/{environment_id}`, `PUT /v1/variable/{variable_id}` (paginated; `value` can be `null` with `undecryptable: true`)

Auth is a bearer JWT; default `base_url` is `http://localhost:8000`. The client only uses the routes listed above; check `../sharenv-api/app/api/v1/*/routes.py` before adding more. Note that private resources are hidden as `404` and `403` means authenticated-but-forbidden (e.g. reader role); `401` is reserved for missing/invalid bearer auth (including expired refresh replay).
