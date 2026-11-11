# Configuration

The global file is at `~/.config/sharenv/config.toml` on Linux. The repository's
root `config.toml` only provides the defaults embedded at build time. Editing
the root file changes the defaults only after a rebuild. Project-specific state
lives in `.sharenv/config.toml` in the current directory.

```toml
config_version = 2
config_dir_name = ".sharenv"
config_file_path = ".sharenv/config.toml"

[api_config]
base_url = "http://localhost:8000"

[api_config.routes]
register = "/v1/auth/register"
login = "/v1/auth/login"
refresh = "/v1/auth/refresh"
logout = "/v1/auth/logout"
verify = "/v1/auth/verify"
resend = "/v1/auth/resend"
invitations = "/v1/invitations"
projects = "/v1/project/"
environments = "/v1/environment"
variables = "/v1/variable"
environment_name_param = "environment_name"

[auth]
token_service = "sharenv"   # keyring service name
token_account = "default"   # keyring account name; the refresh token lives at `<account>-refresh`

[environment_files]
environments = ["dev", "prod", "local"]   # one .env.<name> per entry
env_file_prefix = ".env"
```

Access and refresh tokens are stored in the OS keyring (service/account per
`[auth]`); there is no plaintext token file. Keyring account names are

- **derived per server**: `<token_account>-<fnv1a64(base_url) hex>`, so sessions
  against different servers (dev/prod) never overwrite each other, and
- **or namespaced explicitly** with the per-invocation `--profile <name>` flag.

The refresh token lives at `<slot>-refresh` and a pending verification email
(if any) at `<slot>-pending-verify`. On your first run after upgrading to this
scheme, the CLI sees a new (empty) slot — just log in again.

```bash
sharenv login                       # slot: default-<fnv1a64(base_url)>
sharenv login --profile work        # slot: default-work
sharenv whoami                      # show slot, user, session expiry, API URL
```

Non-HTTPS URLs are rejected unless the host is loopback. Use `--insecure` on a
single command to override that check; it cannot be stored in config or in an
environment variable.

Scalar values can be overridden with environment variables using the `SHARENV`
prefix and `__` as the nesting separator. The assignment goes before the
command and applies to that run only:

```bash
SHARENV__API_CONFIG__BASE_URL=https://sharenv.example.com sharenv login
SHARENV__AUTH__TOKEN_ACCOUNT=work sharenv login
```

Array-valued keys such as `environment_files.environments` cannot be set from
the environment yet; edit them in `config.toml` instead.

On upgrade from an older install, `config.toml` is migrated in place (and a
backup is written to `config.toml.bak`) when the new `config_version` is
incompatible with the embedded schema.
