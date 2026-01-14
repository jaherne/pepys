use crate::models::CommandRecord;
use crate::storage::Storage;
use anyhow::Result;
use std::env;

pub struct Recorder {
    storage: Storage,
}

impl Recorder {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn record(
        &self,
        command: String,
        exit_code: i32,
        duration_ms: i64,
        working_directory: Option<String>,
        output: Option<String>,
    ) -> Result<i64> {
        let cwd = working_directory.unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| String::from("unknown"))
        });

        let record = CommandRecord::new(command, exit_code, duration_ms, cwd, output);

        self.storage.insert(&record)
    }
}

/// Generate shell integration script for bash
pub fn generate_bash_integration() -> String {
    r#"# Pepys shell integration for bash
# Add this to your ~/.bashrc or source it

_pepys_command=""
_pepys_start_time=0

_pepys_preexec() {
    _pepys_command="$1"
    _pepys_start_time=$(date +%s%3N)
}

_pepys_precmd() {
    local exit_code=$?
    if [ -n "$_pepys_command" ]; then
        local end_time=$(date +%s%3N)
        local duration=$(( end_time - _pepys_start_time ))

        # Record the command
        pepys record \
            --command "$_pepys_command" \
            --exit-code $exit_code \
            --duration-ms $duration \
            --working-directory "$PWD"

        _pepys_command=""
    fi
}

# Install hooks
if [ -z "$PROMPT_COMMAND" ]; then
    PROMPT_COMMAND="_pepys_precmd"
else
    PROMPT_COMMAND="_pepys_precmd;$PROMPT_COMMAND"
fi

# For bash, we need to use DEBUG trap for preexec
trap '_pepys_preexec "$BASH_COMMAND"' DEBUG
"#
    .to_string()
}

/// Generate shell integration script for zsh
pub fn generate_zsh_integration() -> String {
    r#"# Pepys shell integration for zsh
# Add this to your ~/.zshrc or source it

_pepys_command=""
_pepys_start_time=0

pepys_preexec() {
    _pepys_command="$1"
    _pepys_start_time=$(date +%s%3N)
}

pepys_precmd() {
    local exit_code=$?
    if [ -n "$_pepys_command" ]; then
        local end_time=$(date +%s%3N)
        local duration=$(( end_time - _pepys_start_time ))

        # Record the command
        pepys record \
            --command "$_pepys_command" \
            --exit-code $exit_code \
            --duration-ms $duration \
            --working-directory "$PWD"

        _pepys_command=""
    fi
}

# Install hooks
autoload -Uz add-zsh-hook
add-zsh-hook preexec pepys_preexec
add-zsh-hook precmd pepys_precmd
"#
    .to_string()
}
