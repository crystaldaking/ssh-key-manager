# SSH Key Manager (skm)

TUI application for managing SSH keys on macOS, Linux, and Windows (including WSL).

## Features

- **Key Management**: List, view, create, edit, and delete SSH keys
- **Interactive Key Generation**: Create new ED25519 or RSA keys with guided wizard
- **Secure Backup/Restore**: Export and import encrypted key backups
- **Cross-Platform**: Works on macOS, Linux, and Windows

## Installation

### From Source

```bash
git clone https://github.com/example/ssh-key-manager
cd ssh-key-manager
cargo build --release
```

The binary will be available at `target/release/skm`.

## Usage

```bash
# Start TUI application
skm

# Use custom SSH directory
skm --ssh-dir /path/to/ssh

# Enable debug logging
skm --debug
```

## Keyboard Shortcuts

### Global
- `Ctrl+H` - Toggle help
- `Ctrl+Q` - Quit

### Key List
- `j`/`↓` - Move down
- `k`/`↑` - Move up
- `Enter` - View key details
- `n` - Create new key
- `e` - Export keys
- `i` - Import keys
- `d` - Delete selected key
- `r` - Refresh list
- `q` - Quit

### Key Detail
- `ESC` - Back to list
- `c` - Edit comment

## License

MIT
