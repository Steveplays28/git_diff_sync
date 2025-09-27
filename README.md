# Git Diff Sync

Synchronize uncommitted Git changes to other devices.

## Installation

### CLI

```bash
cargo install --path crates/git_diff_sync_cli
git diff-sync
```

After running the application once, the user-specific config file will be located at:

- Linux
  - `$XDG_CONFIG_HOME` or `$HOME/.config`
  - Example: `/home/USER_NAME/.local/share/git_diff_sync/config.json`
- MacOS
  - `$HOME/Library/Application Support`
  - Example: `/Users/USER_NAME/Library/Application Support/git_diff_sync/config.json`
- Windows
  - `{FOLDERID_LocalAppData}`
  - Example: `C:\Users\USER_NAME\AppData\Roaming\git_diff_sync\config.json`

See [directories::BaseDirs::config_local_dir](https://docs.rs/directories/latest/directories/struct.BaseDirs.html#method.config_local_dir).

Repository-specific config files can also be used to configure Git Diff Sync, which need to be located in `.git_diff_sync/config.json`. Parent directories will be searched up to the root directory.  
Repository-specific config files have the same format as user-specific config files, though they have higher priority than user-specific config files.

Environment variables can also be used to configure Git Diff Sync, these need to be prefixed with `GIT_DIFF_SYNC_`.

### Server

```bash
cargo install --path crates/git_diff_sync_server
git_diff_sync_server --api-keys="API_KEY_0_HERE","API_KEY_1_HERE"
```

The server will be started on port 9205 by default.

#### Docker Compose

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

#### CLI

```bash
cargo run -- --api-key "admin"
```

#### Server

```bash
cargo run -p git_diff_sync_server -- --api-keys="admin"
```
