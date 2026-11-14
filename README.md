# sharenv

CLI tool to sync, manage, and share project environment variables.
Connects to a self-hosted [sharenv-api](https://github.com/pr4j3sh/sharenv-api)
server to keep environment configurations centralized and accessible.

## Prerequisites

- [sharenv-api](https://github.com/pr4j3sh/sharenv-api) instance
- [Rust 1.85+](https://rust-lang.org/tools/install/) (only for the from-source install)

## Installation

### From crates.io

```bash
cargo install sharenv
sharenv help
```

### Prebuilt binaries

Prebuilt binaries for Linux (x86_64, aarch64) are
attached to every [release](https://github.com/pr4j3sh/sharenv/releases):

```bash
curl -LO https://github.com/pr4j3sh/sharenv/releases/latest/download/sharenv-0.1.2-x86_64-unknown-linux-gnu.tar.gz
curl -LO https://github.com/pr4j3sh/sharenv/releases/latest/download/SHA256SUMS
sha256sum -c SHA256SUMS --ignore-missing
tar -xzf sharenv-0.1.2-x86_64-unknown-linux-gnu.tar.gz
sudo install -m 0755 sharenv /usr/local/bin/sharenv
sharenv help
```

Substitute the archive's version from the release page. The binaries need a
recent glibc; there is no macOS or Windows build (the tool is Unix-only).
Verify the download with `SHA256SUMS` before installing.

### From source

```bash
git clone https://github.com/pr4j3sh/sharenv.git
cd sharenv
cargo build --release        # or: cargo install --path .
./target/release/sharenv help
```

The global config file is `~/.config/sharenv/config.toml` on Linux, created
automatically on first run.

- Edit `base_url` in the config file as your API server's url:

```toml
[api_config]
base_url = "https://sharenv.example.com"
```

- Create a user

```bash
sharenv register
```

## Quick start

```bash
sharenv register          # create an account (or: sharenv login, for returning users)
sharenv init              # project name = current directory name
sharenv push dev          # push .env.dev
sharenv pull dev          # pull it back
```

See the [usage guide](docs/usage.md) for sharing, roles, members, and delete
flows.

## Documentation

- [Usage guide](docs/usage.md)
- [Command reference](docs/commands.md)
- [Configuration](docs/configuration.md)
- [Environment file format](docs/env-format.md)
- [Security & trust model](docs/security.md)
- [Changelog](CHANGELOG.md)

## License

[MIT](LICENSE)
