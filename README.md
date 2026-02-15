# SSH Key Manager (skm)

TUI/CLI application for managing SSH keys on macOS, Linux, and Windows (including WSL).

## Features

- **Key Management**: List, view, create, edit, and delete SSH keys
- **Interactive Key Generation**: Create new ED25519 or RSA keys with guided wizard
- **Secure Backup/Restore**: Export and import encrypted key backups
- **Cross-Platform**: Works on macOS, Linux, and Windows
- **Dual Mode**: TUI for interactive use, CLI for automation (cron, scripts)

## Installation

### From Source

```bash
git clone https://github.com/example/ssh-key-manager
cd ssh-key-manager
cargo build --release
```

The binary will be available at `target/release/skm`.

## Usage

### TUI Mode (Default)

Launch the interactive TUI interface:

```bash
skm
```

### CLI Mode

Use command-line subcommands for automation:

```bash
# List all keys
skm list
skm list --format json
skm list --format names

# Generate a new key
skm generate --key-type ed25519 --filename github_key --comment "GitHub key"
skm generate -t rsa -b 4096 -f work_key -c "Work account"

# Export keys (for cron backups)
skm export -o ~/backups/ssh_$(date +%Y%m%d).skm -p "-"

# Import keys
skm import -f backup.skm -p "my passphrase"

# Show key details
skm show id_ed25519

# Delete a key
skm delete my_key --force
```

## CLI Reference

### Global Options

```
-s, --ssh-dir <PATH>    Path to SSH directory (default: ~/.ssh)
-d, --debug             Enable debug logging
-h, --help              Print help
-V, --version           Print version
```

### Commands

#### `list` - List all SSH keys

```bash
skm list [OPTIONS]

Options:
  -f, --format <FORMAT>  Output format [default: table] [possible values: table, json, names]
```

Examples:
```bash
skm list                    # Table format
skm list -f json            # JSON format
skm list -f names           # Just names, one per line
```

#### `generate` - Generate a new SSH key

```bash
skm generate [OPTIONS]

Options:
  -t, --key-type <TYPE>      Key type [default: ed25519] [possible values: ed25519, rsa]
  -f, --filename <NAME>      Key filename
  -c, --comment <TEXT>       Comment for the key
  -p, --passphrase <PASS>    Passphrase (use '-' for stdin)
  -b, --bits <BITS>          Key bits for RSA [default: 4096]
```

Examples:
```bash
# Generate ED25519 key with defaults
skm generate

# Generate RSA key with custom settings
skm generate -t rsa -b 4096 -f work_key -c "work@company.com"

# Generate with passphrase from stdin
skm generate -f secure_key -p "-"
```

#### `export` - Export keys to encrypted backup

```bash
skm export [OPTIONS] --output <PATH>

Options:
  -o, --output <PATH>        Output file path (required)
  -p, --passphrase <PASS>    Passphrase for encryption (use '-' for stdin)
  -k, --keys <NAMES>         Export only specific keys (can be used multiple times)
      --public-only          Export public keys only
  -d, --description <TEXT>   Description for the backup
```

Examples:
```bash
# Export all keys
skm export -o ~/backup.skm -p "my secure passphrase"

# Export specific keys only
skm export -o ~/github_keys.skm -k id_ed25519_github -k id_rsa_github -p "-"

# Export for cron (passphrase via stdin for security)
echo "passphrase" | skm export -o ~/backups/ssh_$(date +%Y%m%d).skm -p "-"

# Export only public keys
skm export -o ~/public_only.skm --public-only -p "passphrase"
```

#### `import` - Import keys from encrypted backup

```bash
skm import [OPTIONS] --file <PATH>

Options:
  -f, --file <PATH>          Backup file path (required)
  -p, --passphrase <PASS>    Passphrase for decryption (use '-' for stdin)
  -s, --strategy <STRATEGY>  Merge strategy [default: skip] [possible values: skip, overwrite, rename]
      --dry-run              Show what would be imported without actually importing
```

Examples:
```bash
# Import with default settings (skip existing)
skm import -f backup.skm -p "passphrase"

# Import with overwrite
skm import -f backup.skm -p "passphrase" --strategy overwrite

# Dry run to preview
skm import -f backup.skm -p "passphrase" --dry-run
```

#### `delete` - Delete an SSH key

```bash
skm delete [OPTIONS] <NAME>

Arguments:
  <NAME>  Key name to delete

Options:
  -f, --force    Force deletion without confirmation
```

Examples:
```bash
# Delete with confirmation
skm delete old_key

# Delete without confirmation
skm delete old_key --force
```

#### `show` - Show details of a specific key

```bash
skm show <NAME>

Arguments:
  <NAME>  Key name
```

Examples:
```bash
skm show id_ed25519
```

## Automation with Cron

Create a daily backup of your SSH keys:

```bash
# Edit crontab
crontab -e

# Add daily backup at 2 AM
0 2 * * * echo "your_passphrase" | /usr/local/bin/skm export -o ~/backups/ssh_$(date +\%Y\%m\%d).skm -p "-"

# Or use a password file (less secure)
0 2 * * * cat ~/.ssh/backup_passphrase | /usr/local/bin/skm export -o ~/backups/ssh_$(date +\%Y\%m\%d).skm -p "-"
```

## TUI Keyboard Shortcuts

### Global
- `Ctrl+H` - Toggle help
- `Ctrl+Q` - Quit application

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

## Security Notes

- Private keys are encrypted using the modern `age` encryption library
- Passphrases are never logged or stored
- For cron jobs, consider using a passphrase file with restricted permissions (600)
- Exported backups (.skm files) contain both private and public keys - keep them secure

## License

MIT
