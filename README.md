# Pepys

### Currently not working or tested, do not use or trust yet! You will experience data loss
A command history tool that records shell commands with rich metadata. 

## Features

- **Automatic Recording**: Capture every shell command with metadata
  - Command text
  - Exit status
  - Execution duration
  - Working directory
  - Optional output capture

- **Interactive Browser**: TUI for browsing command history
  - Scrollable list with metadata display
  - Multi-selection support
  - Delete commands
  - Keyboard navigation

- **Annotations (to be implemented)**: Add notes to specific commands

- **Export Capabilities**:
  - Combine selected commands into bash scripts
  - Export to markdown with full metadata

- **Statistics**: View insights about your command history

## Installation

```bash
cargo build --release
sudo cp target/release/pepys /usr/local/bin/
```

## Setup

### Bash

Add to your `~/.bashrc`:

```bash
eval "$(pepys init --shell bash)"
```

### Zsh

Add to your `~/.zshrc`:

```bash
eval "$(pepys init --shell zsh)"
```

## Usage

### Recording Commands

Once shell integration is enabled, pepys automatically records all commands. You can also manually record a command:

```bash
pepys record --command "echo hello" --exit-code 0 --duration-ms 150
```

### Browsing History

Launch the interactive TUI browser:

```bash
pepys browse
```

Keyboard shortcuts:
- `↑/k` - Move up
- `↓/j` - Move down
- `Space` - Toggle selection
- `a` - Annotate selected command (to be implemented)
- `d` - Delete selected command
- `?` - Toggle help
- `q` - Quit

### List Recent Commands

```bash
pepys list --limit 20
```

### Export Commands

Export to bash script:
```bash
pepys export-script 1 2 3 --output script.sh
```

Export to markdown:
```bash
pepys export-markdown 1 2 3 --output commands.md
```

### View Statistics

```bash
pepys stats
```

## Data Storage

Command history is stored in a SQLite database located at:
- macOS: `~/Library/Application Support/pepys/history.db`
- Linux: `~/.local/share/pepys/history.db`
- Windows: `C:\Users\<username>\AppData\Roaming\pepys\history.db`

## Future Features

The following features are planned for future releases:

- **Annotations**: Add free-form text annotations to commands to highlight important notes
- **Query System**: Advanced filtering of command history
  - Complex boolean queries
  - Search for previous commands based on duration, execution status, executing user, etc
- **Cloud Sync**: Synchronize history across machines and users
- **Output Capture**: Configurable automatic capture of command output
