# Git Diff Sync

Synchronize uncommitted Git changes to other devices.

## Installation

CLI:

```bash
cargo install --path crates/git_diff_sync_cli
git diff-sync
```

Server:

```bash
cargo install --path crates/git_diff_sync_server
git_diff_sync_server --api-keys="API_KEY_0_HERE","API_KEY_1_HERE"
```

The server will be started on port 9205 by default.

### Docker Compose

See [docker-compose.yaml](docker-compose.yaml).

## Development

### Prerequisites

- [Rustup and Cargo](https://www.rust-lang.org/tools/install)
- rustc v1.90.0-nightly or newer
- Clang and Mold (Linux/MacOS)
  - See [.cargo/config.toml](.cargo/config.toml)
- LLD (Windows)
  - See [.cargo/config.toml](.cargo/config.toml)

### Building

```bash
cargo build
```

### Running in development environments

CLI:

```bash
cargo run -- --api-key "admin"
```

Server:

```bash
cargo run -p git_diff_sync_server -- --api-keys="admin"
```
