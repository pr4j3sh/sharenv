# sharenv

A command-line tool to securely sync, manage, and share project environment variables. Connects to a self-hosted [sharenv-api](https://github.com/pr4j3sh/sharenv-api) server to keep environment configurations centralized and accessible.

It uses [`pr4j3sh/sharenv-api`](https://github.com/pr4j3sh/sharenv-api) server.

## Prerequisites

- [Rust 1.85+](https://rust-lang.org/tools/install/)
- [`pr4j3sh/sharenv-api`](https://github.com/pr4j3sh/sharenv-api)

## Installation

### Using Source

- Clone the repository and build it:

```bash
git clone https://github.com/pr4j3sh/sharenv.git
cd sharenv
cargo build --release
```

> The binary is written to `target/release/sharenv`:

```bash
./target/release/sharenv help
```

- To install it on your `PATH` instead:

```bash
cargo install --path .
```

> **Important:** the global `config.toml` lives in your OS config directory
> (`~/.config/sharenv/config.toml` on Linux) and is created automatically from
> the defaults embedded at build time. Editing the repository's root
> `config.toml` only changes those defaults after a rebuild. Project-specific
> state lives in `.sharenv/config.toml` in the current directory.

### Setting Up Backend

- Follow the instructions to setup [`pr4j3sh/sharenv-api`](https://github.com/pr4j3sh/sharenv-api)

- Edit the global `config.toml` (or set the environment override below) and
  change `base_url`:

```toml
[api_config]
base_url = "https://sharenv.example.com"
```

- or override it with an environment variable:

```bash
SHARENV__API_CONFIG__BASE_URL=https://sharenv.example.com sharenv login
```

## Usage

### 1. Create an account or log in

```bash
sharenv register   # r
sharenv login      # l
sharenv logout
```

If the server has email verification enabled, `register` emails you a one-time
code and asks for it before logging you in. Verify later (or on another
machine) with `sharenv verify [email]`; `sharenv resend` sends a new code.
Sessions use short-lived access tokens with a rotating refresh token; the CLI
refreshes automatically. `sharenv logout` revokes your session server-side and
removes the local tokens.

### 2. Initialize a project

Run this from inside the project you want to track:

```bash
sharenv init                 # project name = current directory name
sharenv init -p my-project   # or choose one explicitly
```

### 3. Push variables

Push one environment:

```bash
sharenv push -e dev
```

Or push every remote environment that has a matching `.env.<env>` file:

```bash
sharenv push
```

New variables are created. If a variable already exists remotely with a
different value, `push` asks whether to update the remote one. Values are
hidden by default; pass `--show-values` to print sanitized values.

`push` shows the destination project and owner first. Pushing to a project you
do not own always asks for confirmation.

### 4. Share the project or add an environment

Only the project owner can share it. Sharing uses invitations: the invited
teammate must accept before they gain access. Run this from the initialized
project directory:

```bash
sharenv invite teammate@example.com
```

You'll be asked for the role (`writer` or `reader`, default `reader`), which
environments to grant, and your password (the API re-authenticates you before
creating the invitation). If the server has no mail provider configured, the
invitation token is printed once — send it to your teammate out-of-band.

The invitee accepts with:

```bash
sharenv invites accept <token>
```

Manage pending invitations for the current project:

```bash
sharenv invites list
sharenv invites cancel <invitation_id>
```

Use `-e` to create another environment for the current project:

```bash
sharenv add -e staging
```

This creates the environment remotely and adds an empty `.env.staging` file if
it does not exist yet.

`invites accept` prints the project id, and your teammate can then pull the
variables without a local `.sharenv` config:

```bash
sharenv pull --pid <project_id> -e dev
```

### 5. Pull variables

Pull one environment:

```bash
sharenv pull -e dev
```

Run `sharenv pull` without `-e` to pull all remote environments. By default
`pull` reads the project id from `.sharenv/config.toml`. Pass
`--pid <project_id>` to pull a shared project from any directory; without `-e`
that pulls all of its environments too.

You must be a member of the project to pull it. The CLI resolves the project
from the server first and refuses non-members before fetching any variables.
Values are hidden by default; pass `--show-values` to print them.

### 6. List projects or environments

```bash
sharenv list projects       # ls p
sharenv list environments   # ls e
```

`list environments` uses the project id from `.sharenv/config.toml`.

### 7. Delete a project or environment

Run this from the initialized project directory. All commands ask for
confirmation; add `--yes` (or `-y`) to skip the prompt.

```bash
sharenv delete                   # current project (id from .sharenv/config.toml)
sharenv delete -p my-project     # project by name
sharenv delete -e dev            # one environment
```

Deleting the current project uses the id from `.sharenv/config.toml` and removes
that local config on success. `-p/--project` deletes by name through the API; the
local config is removed only if the deleted project id matches. Local
`.env.<env>` files are left untouched.

### 8. Help

```bash
sharenv help      # h
sharenv --help    # -h
```

## Security

`sharenv` is not end-to-end encrypted: values are sent to the API, which
decrypts them, and local `.env.*` files are plaintext. Read
[`docs/security.md`](docs/security.md) for the full trust model and limitations.

Key behaviors:

- Non-loopback `http://` URLs are refused unless you pass `--insecure` for that
  invocation. The default `http://localhost:8000` is allowed for local
  development.
- `push`/`pull` hide values by default. `--show-values` prints sanitized values
  for that run.
- The project name and owner are shown before transfers. Pushing to a project
  you do not own requires confirmation.
- `pull` requires project membership and refuses non-members before fetching
  variables.

## Reference

### Commands

| Command    | Alias               | Flags                                                    | Description                                                                                 |
| ---------- | ------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| `register` | `r`                 | —                                                        | Register a new account.                                                                     |
| `login`    | `l`                 | —                                                        | Log in.                                                                                     |
| `logout`   | —                   | —                                                        | Revoke the server-side session and remove local tokens.                                      |
| `verify`   | —                   | `[email]`                                                | Verify an account with the emailed code (defaults to the email saved at registration). Saves the session on success. |
| `resend`   | —                   | `[email]`                                                | Resend the verification code.                                                               |
| `init`     | `i`                 | `-p`, `--project <project>`, `--adopt`                   | Create the remote project and local `.env` files. Defaults to the current directory's name. `--adopt` records identity for an existing project. |
| `invite`   | —                   | `<email>`, `--role <writer\|reader>`                     | Invite a user to the current project (owner only; re-authenticates with your password). |
| `invites`  | —                   | `list`, `cancel <id>`, `accept <token>`                   | List, cancel (owner, re-authenticates) or accept invitations. Accepting requires the invited account. |
| `add`      | `a`                 | `-e`, `--environment <env>`                              | Create an environment for the current project.                                               |
| `push`     | —                   | `-e`, `--environment <env>`, `--show-values`             | Push one environment, or all environments with local `.env.<env>` files when `-e` is omitted. Values are hidden unless `--show-values`. |
| `pull`     | —                   | `-e`, `--environment <env>`, `-pid`, `--project_id <id>`, `--show-values` | Pull one environment, or all environments when `-e` is omitted. Requires project membership. |
| `list`     | `ls`                | `projects`, `environments`                               | List your projects or the current project's environments.                                   |
| `delete`   | `d`                 | `-p`, `--project <project>`, `-e`, `--environment <env>`, `-y`, `--yes` | Delete the current project, a project by name, or an environment.                    |
| `help`     | `h`, `--help`, `-h` | —                                                        | Show help.                                                                                  |

Flags accept both space-separated and `=` forms: `-e dev`, `--environment dev`,
`--environment=dev`, `-e=dev`, `-p my-project`, `--project=my-project`,
`--pid=<project_id>`.

### Configuration (`config.toml`)

The global file is at `~/.config/sharenv/config.toml` on Linux. The repository's
root `config.toml` only provides the defaults embedded at build time.

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
`[auth]`). The refresh token lives at `token_account-refresh` and a pending
verification email (if any) at `token_account-pending-verify`; there is no
plaintext token file.

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

### Environment file format

Parsing supports:

```dotenv
# comments are preserved on pull
KEY=value
QUOTED="value with spaces"
SINGLE='literal #value'
export EXPORTED=value
INLINE=value # trailing comment
```
