# sharenv — agent notes

Rust 2024 single-binary CLI that syncs project `.env` files with a remote sharenv-api. No workspace, no CI, no formatter/lint configs.

## Commands

```bash
cargo check --all-targets   # fastest signal
cargo build
cargo run -- <command> [flags]
```

- Aliases: `register`/`r`, `login`/`l`, `init`/`i`, `add`/`a`, `list`/`ls`, `delete`/`d`, `help`/`h`/`--help`/`-h`.
- Flags: `init [-p|--project]`, `add [-u|--user | -e|--environment]`, `push|pull [-e|--environment]`, `pull [--pid|--project_id]`, `delete [-p|--project <name> | -e|--environment <env>] [-y|--yes]`. Both `-e dev` and `--environment=dev` work.
- Run project commands (`init`, `push`, `pull`, `add`, `delete`, `list environments`) from the target project directory. This checkout may contain gitignored `.sharenv/config.toml` and `.env.*`; running from the repo root targets those.

## Configuration: two different files

- **Global config**: `AppConfig::load` uses the OS config dir (`~/.config/sharenv/config.toml` on Linux) and auto-creates it from the root `config.toml` embedded via `include_str!` when missing. The root `config.toml` is compiled in, not read at runtime; editing it requires a rebuild.
- Overrides use `SHARENV` + `__`, e.g. `SHARENV__API_CONFIG__BASE_URL=...`. Array values (e.g. `environment_files.environments`) cannot be overridden by env vars.
- **Local project config**: `./.sharenv/config.toml` containing `[project] id = "..."`. Its directory/filename come from global `config_dir_name` / `config_file_path` and are resolved relative to the cwd. `.sharenv/` and `.env*` are gitignored.
- README is stale on config: it claims `config.toml` is loaded from the cwd and places `config_dir_name`/`config_file_path` under `[environment_files]`. Trust `src/settings/models.rs`, `src/settings/paths.rs`, and the root `config.toml`.
- Schema changes need a version bump in `src/settings/version.rs` plus a step in `src/settings/migrations.rs`; migrations back up to `config.toml.bak`.

## Architecture and conventions

- Entry flow: `src/main.rs` -> `cli::run_cli` -> `commands::*`. Dispatch is hand-rolled on `argv[1]` (no clap); add subcommands in both `src/cli.rs` and `src/commands/mod.rs`.
- Command flags are parsed by `src/utils/args.rs`; the dispatcher ignores extra args itself but passes them to the command.
- `src/api/client.rs` builds a per-command `ApiClient`. It starts without a token; call `.with_token(token)`. Auth calls omit bearer auth, all project/environment/variable calls attach it.
- Non-2xx responses become `ApiError` in `src/utils/error.rs`, surfacing the backend's `detail`; `409 Conflict` is used as "already exists" (push skips those).
- Tokens live in the OS keyring using `[auth]` service/account, not in a file. Login/token reads can fail in headless environments without a secret service.
- `_id` fields in `src/models.rs` intentionally match the backend JSON (`_id`); do not rename to `id`. Variable values accept JSON strings or other scalars.

## Behavior gotchas

- `init` uses the cwd basename unless `-p/--project` is given, validates `[a-z0-9_-]+`, always creates a new remote project, saves its id locally, then creates every `environment_files.environments` entry remotely and as `.env.<env>` locally. If the local project id already matches a remote project, it reports “already initialized” and exits without creating anything.
- `push -e <env>` creates missing variables and prompts to update existing ones whose remote value differs. It never deletes remote-only variables. Without `-e`, it prompts and pushes every remote environment that has a local `.env.<env>` file. `pull -e <env>` prompts per difference and preserves existing comments/formatting; without `-e`, it prompts and pulls every remote environment.
- `pull --pid <project_id>` works without a local `.sharenv` config (for shared projects); otherwise it needs the local project id.
- `add -u/--user` shares the current project; `add -e/--environment` creates that environment in the local project and an empty `.env.<env>` if missing. A `409 Conflict` (environment already exists) is reported as a no-op success.
- `delete` with no target deletes the current project by the id in `.sharenv/config.toml`; `delete -p/--project <name>` deletes by name through the API and removes the local config only when the deleted id matches; `delete -e <env>` resolves the environment within the local project. All prompt unless `--yes`/`-y`; local `.env.*` files are never deleted.
- `list` requires a subject: `list projects` or `list environments` (short aliases `p`/`e`).
- Project/environment names must be lowercase ASCII letters, digits, `-`, or `_`. Env files are `<env_file_prefix>.<env>` (default `.env.dev`). The parser supports comments, single/double quotes, `export`, and inline comments, and rejects duplicate keys.

## Backend API

The sibling FastAPI repo `../sharenv-api` is the contract source of truth; authoritative routes are in `app/api/v1/*/routes.py`. Routes and the environment query param come from `[api_config.routes]`, not hardcoded paths. Current client calls:

- `POST /v1/auth/register`, `POST /v1/auth/login` -> `{ "token": ... }`
- `GET|POST|DELETE /v1/project/`, plus `PUT|DELETE /v1/project/{project_id}` (`PUT` adds a user) and `DELETE /v1/project/name/{project_name}`
- `GET|POST /v1/environment/{project_id}`, `DELETE /v1/environment/{environment_id}` (GET accepts `?environment_name=`)
- `GET|POST /v1/variable/{environment_id}`, `PUT /v1/variable/{variable_id}`

Auth is a bearer JWT; default `base_url` is `http://localhost:8000`. The client only uses the routes listed above; check `../sharenv-api/app/api/v1/*/routes.py` before adding more.
