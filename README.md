# x - Executable Manager

**x** is a simple yet powerful command-line tool for managing multiple executables and versions. It helps you organize, switch between, and run different versions of your executables through an intuitive group-based system.

## 🌟 Features

- **Group Management**: Organize executables into logical groups (e.g., development, production, testing)
- **Quick Switching**: Instantly switch between different executable groups
- **Symlink Management**: Automatically creates and manages symlinks for your executables
- **Smart Discovery**: Add entire directories of executables at once
- **Search & Filter**: Quickly find executables across all groups
- **Enable/Disable**: Control which executables are active without removing them
- **Shell Integration**: Automatically configures your shell PATH

## 📦 Installation

### Building from Source

```bash
# Clone the repository
git clone https://github.com/dwpeng/x.git
cd x

# Build with cargo
cargo build --release

# The binary will be at target/release/x
# Optionally, install it to your cargo bin directory
cargo install --path .
```

## 🚀 Quick Start

### 1. Initialize

First, initialize the configuration file:

```bash
x init
```

This creates a config file at `~/.config/x/config.json` and sets up the bin directory at `~/.local/bin/x`. It will also automatically add this directory to your PATH in your shell configuration.

### 2. Add Executables

Add a single executable:

```bash
# Add with automatic name detection
x add /path/to/executable

# Add with custom alias
x add /path/to/executable -n myalias

# Add to a specific group
x add /path/to/executable -g production
```

Add all executables from a directory:

```bash
x add /path/to/bin/directory
```

### 3. List Your Executables

```bash
# List executables in the active group
x list

# List all groups and their executables
x list --all
```

Output example:
```
* global-default-group-name
  1. node -> /usr/local/bin/node
  2. npm -> /usr/local/bin/npm
  production
  1. app -> /opt/production/app
```

### 4. Switch Between Groups

```bash
# Switch to a different group
x switch production

# Or use the short alias
x s production
```

When you switch groups, **x** automatically updates the symlinks so only executables from the active group are accessible.

### 5. Run Executables

```bash
# Run from the active group
x run myapp arg1 arg2

# Or use the short alias
x r myapp arg1 arg2

# Run from a specific group
x run -g production myapp

# Run with absolute path (bypasses group system)
x run /absolute/path/to/executable
```

You can also run executables directly if you've configured your PATH:
```bash
myapp arg1 arg2
```

## 📖 Command Reference

### Core Commands

#### `init` - Initialize Configuration
```bash
x init              # Create config file
x init -f           # Force re-initialize (creates backup)
```

#### `add` - Add Executables
```bash
x add <path>                    # Add executable or directory
x add <path> -n <name>          # Add with custom name
x add <path> -g <group>         # Add to specific group
```

#### `list` / `ls` - List Executables
```bash
x list              # Show active group
x list --all        # Show all groups
x ls -a             # Short form
```

#### `run` / `r` - Run Executables
```bash
x run <name> [args...]          # Run from active group
x r <name> [args...]            # Short form
x run -g <group> <name>         # Run from specific group
```

#### `switch` / `s` - Switch Groups
```bash
x switch <group>    # Switch to group
x s <group>         # Short form
```

### Management Commands

#### `rm` - Remove Executables
```bash
x rm -n <name>              # Remove by name from active group
x rm -n <name> -g <group>   # Remove from specific group
x rm -g <group> -d          # Delete entire group (with confirmation)
```

#### `rename` - Rename Aliases
```bash
x rename <old> <new>            # Rename in active group
x rename <old> <new> -g <group> # Rename in specific group
```

#### `enable` / `disable` - Control Executables
```bash
x enable <name>             # Enable executable
x disable <name>            # Disable executable
x enable <name> -g <group>  # Enable in specific group
```

#### `info` - Show Details
```bash
x info <name>               # Show info for active group
x info <name> -g <group>    # Show info for specific group
```

#### `search` - Find Executables
```bash
x search <query>            # Search by name or path
```

## 💡 Use Cases

### Managing Node.js Versions

```bash
# Add different Node.js versions to different groups
x add /usr/local/node-14/bin -g node14
x add /usr/local/node-16/bin -g node16
x add /usr/local/node-18/bin -g node18

# Switch between versions
x switch node16
node --version  # Shows v16.x.x

x switch node18
node --version  # Shows v18.x.x
```

### Development vs Production Tools

```bash
# Development group
x add ~/dev-tools/bin -g dev
x add ~/debuggers/gdb -g dev

# Production group
x add /opt/production/bin -g prod

# Work on development
x switch dev

# Deploy to production
x switch prod
```

### Managing Custom Scripts

```bash
# Add your custom scripts
x add ~/my-scripts

# Run them from anywhere (after PATH is configured)
backup-db
deploy-app

# Or use the x run command
x run backup-db
x run deploy-app
```

## 🔧 Configuration

The configuration file is stored at `~/.config/x/config.json`:

```json
{
  "active-group": "global-default-group-name",
  "bin-dir": "/home/user/.local/bin/x",
  "groups": {
    "global-default-group-name": {
      "index": 0,
      "bins": {
        "myapp": {
          "name": "myapp",
          "path": "/usr/local/bin/myapp",
          "source-dir": null,
          "enabled": true
        }
      }
    }
  }
}
```

### Configuration Locations

- **Config file**: `~/.config/x/config.json`
- **Bin directory**: `~/.local/bin/x` (symlinks created here)
- **Backup on force init**: `~/.config/x/config.json.bak`

## 🎯 Tips & Tricks

1. **Use descriptive group names**: Instead of "group1", use names like "dev", "prod", "test"
2. **Leverage enable/disable**: Keep executables configured but disabled when not in use
3. **Search is your friend**: Use `x search` to quickly find executables across all groups
4. **Check info before running**: Use `x info` to verify paths and status
5. **List regularly**: Use `x list -a` to see your complete setup

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## 📄 License

This project is open source. Please check the repository for license details.

## 🔗 Links

- **Repository**: https://github.com/dwpeng/x
- **Issues**: https://github.com/dwpeng/x/issues

## ❓ FAQ

**Q: What happens when I switch groups?**  
A: All symlinks in the bin directory are updated to point to executables from the new active group.

**Q: Can an executable be in multiple groups?**  
A: Yes! You can add the same executable to different groups with different aliases.

**Q: What if an executable name conflicts?**  
A: Use custom names with the `-n` flag when adding to avoid conflicts.

**Q: How do I remove x completely?**  
A: Remove the config directory (`~/.config/x`), bin directory (`~/.local/bin/x`), and remove the PATH entry from your shell config file.

**Q: Does x support Windows?**  
A: Yes! x has cross-platform support for Windows, macOS, and Linux.

---

Made with ❤️ by the x community
