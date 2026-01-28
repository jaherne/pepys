# Pepys

### Currently not working or tested, do not use or trust yet! You will experience data loss
A command history tool that records shell commands with rich metadata. 

## Features

- **Automatic Recording**: Capture every shell command with metadata
  - Command text
  - Exit status
  - Execution duration
  - Working directory
  - Environment variables (configurable)
  - Optional output capture

- **Interactive Browser**: TUI for browsing command history
  - Scrollable list with metadata display
  - Multi-selection support
  - Delete commands
  - Keyboard navigation

- **Annotations**: Add notes to specific commands

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

Add to your `~/.zshrc`:

```bash
eval "$(pepys init)"
```

## Usage

### Command Recording

Once shell integration is enabled, pepys automatically records all commands.

### Output Recording (WIP)

Run the `pepys record` command to enable/disable recording the output of shell commands:

```bash
pepys record
```

When enabled, you'll see a red **[●]** indicator in your prompt. When visible, pepys is recording the output of all commands you run!

Note that this can include sensitive info, such as if you run `cat /etc/shadow`. Be careful, or delete commands from your history with sensitive outputs. 

Output recording is **disabled by default**.

### Environment Variable Capture

Pepys can capture specified environment variables with each command. This is useful for tracking context like virtual environments, AWS profiles, or Node.js environments.

Create or edit `~/.pepys/config.toml`:

```toml
capture_env_vars = ["VIRTUAL_ENV", "AWS_PROFILE", "NODE_ENV"]
```

Captured environment variables are displayed in:
- The interactive browser's details pane
- The `pepys list` output
- Exported markdown and bash scripts

**Limitation:** Only environment variables set in the shell environment are captured (e.g., via `export` or by activating a virtual environment). Inline variable assignments like `FOO=bar command` are not captured as structured data because they only exist in the subprocess environment. However, they remain visible in the command text itself.

### Manual Command Entry

You can manually add a command to history:

```bash
pepys add --command "echo hello" --exit-code 0 --duration-ms 150
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
- `a` - Annotate selected command
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
pepys export 1 2 3 --output script.sh --format script
```

Export to markdown:
```bash
pepys export 1 2 3 --output commands.md --format markdown
```

The `--format` flag defaults to `script` if not specified.

### View Statistics

```bash
pepys stats
```

## Configuration

Pepys can be configured via `~/.pepys/config.toml`. All settings are optional with sensible defaults.

```toml
# Maximum number of commands to store (default: 10000)
# Oldest commands are deleted when limit is exceeded
max_commands = 10000

# Environment variables to capture with each command (default: none)
capture_env_vars = ["VIRTUAL_ENV", "AWS_PROFILE", "NODE_ENV"]
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

## FAQs
### Is this a audit tool? 

No, this is meant for personal consumption. No guarantees are made about data safety, and there's no way to prevent a user from deleting their own data. Do not rely on this for legal auditing.
