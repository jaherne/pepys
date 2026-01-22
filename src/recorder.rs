use crate::config::Config;
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
pub fn generate_bash_integration(_config: &Config) -> String {
    r#"# Pepys shell integration for bash
# Add this to your ~/.bashrc or source it

_pepys_command=""
_pepys_start_time=0
_pepys_original_ps1="$PS1"

# Get current time in milliseconds (cross-platform)
_pepys_get_time_ms() {
    # Try different methods depending on OS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS: use python or perl as fallback
        python3 -c 'import time; print(int(time.time() * 1000))' || \
        perl -MTime::HiRes=time -e 'printf("%.0f\n", time() * 1000)' || \
        echo $(($(date +%s) * 1000))
    else
        # Linux: use date with nanoseconds
        echo $(($(date +%s%N) / 1000000))
    fi
}

# Update prompt with recording indicator
_pepys_update_prompt() {
    local indicator=$(pepys variable recording_indicator --shell bash)
    PS1="${indicator}$_pepys_original_ps1"
}

_pepys_preexec() {
    # Skip internal pepys functions to avoid overwriting the actual command
    [[ "$1" == _pepys_* ]] && return
    _pepys_command="$1"
    _pepys_start_time=$(_pepys_get_time_ms)
}

_pepys_precmd() {
    local exit_code=$?

    # Update prompt indicator
    _pepys_update_prompt

    if [ -n "$_pepys_command" ]; then
        local end_time=$(_pepys_get_time_ms)
        local duration=$(( end_time - _pepys_start_time ))

        # Always record the command
        pepys add \
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
pub fn generate_zsh_integration(_config: &Config) -> String {
    r#"# Pepys shell integration for zsh
# Add this to your ~/.zshrc or source it

_pepys_command=""
_pepys_start_time=0

# Get current time in milliseconds (cross-platform)
_pepys_get_time_ms() {
    # Try different methods depending on OS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS: use python or perl as fallback
        python3 -c 'import time; print(int(time.time() * 1000))' || \
        perl -MTime::HiRes=time -e 'printf("%.0f\n", time() * 1000)' || \
        echo $(($(date +%s) * 1000))
    else
        # Linux: use date with nanoseconds
        echo $(($(date +%s%N) / 1000000))
    fi
}

# Get recording indicator for prompt
_pepys_prompt_indicator() {
    pepys variable recording_indicator --shell zsh
}

pepys_preexec() {
    _pepys_command="$1"
    _pepys_start_time=$(_pepys_get_time_ms)
}

pepys_precmd() {
    local exit_code=$?

    if [[ -n "$_pepys_command" ]]; then
        local end_time=$(_pepys_get_time_ms)
        local duration=$(( end_time - _pepys_start_time ))

        # Always record the command
        pepys add \
            --command "$_pepys_command" \
            --exit-code $exit_code \
            --duration-ms $duration \
            --working-directory "$PWD"

        _pepys_command=""
    fi
}

# Set up prompt with recording indicator (appended to the end)
# This uses command substitution that runs every time the prompt is displayed
setopt PROMPT_SUBST
PROMPT="$PROMPT"'$(_pepys_prompt_indicator)'

# Install hooks
autoload -Uz add-zsh-hook
add-zsh-hook preexec pepys_preexec
add-zsh-hook precmd pepys_precmd
"#
    .to_string()
}
