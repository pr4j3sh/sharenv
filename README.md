# sharenv

A command-line tool to sync, manage, and share project environment variables.
Connects to a self-hosted [sharenv-api](https://github.com/pr4j3sh/sharenv-api)
server to keep environment configurations centralized and accessible.

## Prerequisites

- [Rust 1.85+](https://rust-lang.org/tools/install/)
- A running [sharenv-api](https://github.com/pr4j3sh/sharenv-api) instance (its
  own setup instructions apply)

## Installation

```bash
git clone https://github.com/pr4j3sh/sharenv.git
cd sharenv
cargo build --release        # or: cargo install --path .
./target/release/sharenv help
```

The global config file is `~/.config/sharenv/config.toml` on Linux, created
automatically on first run. Point `base_url` at your server and log in:

```toml
[api_config]
base_url = "https://sharenv.example.com"
```

```bash
SHARENV__API_CONFIG__BASE_URL=https://sharenv.example.com sharenv login
```

## Quick start

```bash
sharenv register   # or: sharenv login
sharenv init              # project name = current directory name
sharenv push dev          # push .env.dev
sharenv pull dev          # pull it back
```

See the [usage guide](docs/usage.md) for sharing, roles, members, and delete
flows.

## Documentation

- [Usage guide](docs/usage.md) — accounts, sessions, init, push/pull, sharing, members, delete
- [Command reference](docs/commands.md) — all commands and flags
- [Configuration](docs/configuration.md) — `config.toml` layout, overrides, keyring storage
- [Environment file format](docs/env-format.md)
- [Security & trust model](docs/security.md)
