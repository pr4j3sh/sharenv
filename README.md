# sharenv

> manage and share `.env` variables securely

`sharenv` is a cli tool that allows you to sync project `.env` files to a
remote server and share them securely with others.
It uses [`pr4j3sh/sharenv-api`](https://github.com/pr4j3sh/sharenv-api) to manage this.

## Prerequisites

- [Rust 1.85+](https://rust-lang.org/tools/install/)
- [`pr4j3sh/sharenv-api`](https://github.com/pr4j3sh/sharenv-api)

## Installation

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

> **Important:** `config.toml` is loaded from the **current working directory**.
> Run the binary from the cloned repository (which ships a `config.toml`), or
> place your own `config.toml` in whatever directory you run it from.

### Setting Up Backend

- Follow the instructions to setup [`pr4j3sh/sharenv-api`](https://github.com/pr4j3sh/sharenv-api)

- Edit `config.toml` and change `base_url`:

```toml
[api_config]
base_url = "https://sharenv.example.com"
```

or override it with an environment variable:

```bash
SHARENV__API_CONFIG__BASE_URL=https://sharenv.example.com sharenv login
```

## Usage

### 1. Create an account or log in

```bash
sharenv register   # r
sharenv login      # l
```

### 2. Initialize a project

Run this from inside the project you want to track:

```bash
sharenv init                 # project name = current directory name
sharenv init -n my-project   # or choose one explicitly
```

### 3. Push variables

Fill in an env file, then upload it:

```bash
sharenv push -e dev
```

### 4. Share the project

Only the project owner can share it. Run this from the initialized project
directory:

```bash
sharenv add --user teammate@example.com
```

The command prints the project id returned by the API. Send that id to your
teammate, and they can pull the variables without a local `.sharenv` config:

```bash
sharenv pull --pid <project_id> -e dev
```

### 5. Pull variables

```bash
sharenv pull -e dev
```

By default `pull` reads the project id from `.sharenv/config.toml`. Pass
`--pid <project_id>` to pull a project shared with you from any directory.

### 6. Help

```bash
sharenv help      # h
sharenv --help    # -h
```

## Reference

### Commands

| Command    | Alias               | Flags                                                    | Description                                                                                 |
| ---------- | ------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| `register` | `r`                 | —                                                        | Register a new account.                                                                     |
| `login`    | `l`                 | —                                                        | Log in.                                                                                     |
| `init`     | `i`                 | `-n`, `--name <project>`                                 | Create the remote project and local `.env` files. Defaults to the current directory's name. |
| `add`      | `a`                 | `-u`, `--user <email>`                                   | Share the current project with a user (owner only).                                         |
| `push`     | —                   | `-e`, `--environment <env>`                              | Upload variables from `.env.<env>` to the server.                                           |
| `pull`     | —                   | `-e`, `--environment <env>`, `-pid`, `--project_id <id>` | Download variables into `.env.<env>`, with confirmation prompts.                            |
| `help`     | `h`, `--help`, `-h` | —                                                        | Show help.                                                                                  |

Flags accept both space-separated and `=` forms: `-e dev`, `--environment dev`,
`--environment=dev`, `-e=dev`, `--pid=<project_id>`.

### Configuration (`config.toml`)

```toml
[api_config]
base_url = "http://localhost:8000"

[api_config.routes]
register = "/v1/auth/register"
login = "/v1/auth/login"
projects = "/v1/project/"
environments = "/v1/environment"
variables = "/v1/variable"
environment_name_param = "environment_name"

[auth]
token_service = "sharenv"   # keyring service name
token_account = "default"   # keyring account name

[environment_files]
environments = ["dev", "prod", "local"]   # one .env.<name> per entry
config_dir_name = ".sharenv"
config_file_path = ".sharenv/config.toml"
env_file_prefix = ".env"
```

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
