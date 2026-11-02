# sharenv — agent notes

Rust CLI (single binary crate, no workspace) that stores project `.env` variables in a remote backend.

## Current state: mid-migration, does not compile

The working tree is being rewritten from direct MongoDB access to an HTTP API, while `HEAD` is the old MongoDB version. Do not trust `HEAD` for current behavior, and expect `cargo check` to fail until the rewrite lands.

Verified breakpoints:

- `reqwest` is missing the `json` feature, so `.json()` calls fail in `src/api/auth.rs` and `src/api/project.rs`.
- `mongodb` was removed from `Cargo.toml` but is still imported by `commands/init.rs`, `commands/push.rs`, and `utils/env_sync.rs`.
- `models.rs` no longer exports `Environment`, `Project`, or `EnvironmentVariable`, but the old command/util code still imports them.
- `src/cli.rs` passes an undefined `client` to command handlers.
- `src/api/environment.rs`, `src/api/variable.rs`, and `src/utils/token.rs` are empty placeholders.
- `src/client.rs` is not declared in `main.rs`, so it is not compiled.
- `keyring` is a dependency but unused; token persistence is not implemented.

## Commands

```bash
cargo check --all-targets   # fastest signal, currently expected to fail
cargo build
cargo run -- init           # or: add / push / pull / help
```

There are no tests, CI workflows, formatter configs, or lint configs. `cargo test` has nothing to run yet.

## Configuration

- `AppConfig::load` (`src/settings.rs`) reads `config.toml` relative to the **current working directory**, and silently skips it if absent. Run from the repo root (or wherever the intended `config.toml` lives).
- Env overrides use the `SHARENV` prefix and `__` as the nesting separator, e.g. `SHARENV__API_CONFIG__BASE_URL`.
- `config.toml` is currently out of sync with the structs: the file defines `[api]` and `[default]`, while `AppConfig` expects `api_config` and `environment_files`. Loading fails until one side is renamed.
- `_id` fields in `src/models.rs` intentionally match the API's JSON id; do not rename them to `id`.

## CLI and project identity

- Dispatch is hand-rolled in `src/cli.rs` using only `argv[1]`; there is no clap and extra args are ignored. Add new subcommands in `src/cli.rs` plus `src/commands/mod.rs`.
- Aliases: `init`/`i`, `add`/`a`, `help`/`h`. Only `help` works; the printed suggestion `--help` actually falls through to "not a valid command". `add` and `pull` are stubs.
- `init` and `push` use the current directory's basename as the project name, so run them from the project directory.
- Environment files must be named `.env.<env>`; `get_env_name` strips the `.env.` prefix. `.env*` files are gitignored.

## Backend API contract

The backend is the sibling repo `../sharenv-api` (FastAPI, GitHub `pr4j3sh/sharenv-api`). Authoritative routes live in `app/api/v1/*/routes.py`:

- `POST /v1/auth/register`, `POST /v1/auth/login` → `{ "token": ... }`
- `GET|POST /v1/project/`, `PUT /v1/project/{project_id}`
- `GET|POST /v1/environment/{project_id}`
- `GET|POST /v1/variable/{environment_id}`
- Auth is a bearer JWT; project names are validated as `[a-z0-9_-]+`.

The Rust client currently calls `{base_url}/register` and `{base_url}/api/v1/projects`, which do not match the backend. Check the backend routes before adding API calls.
