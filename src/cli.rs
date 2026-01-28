use clap::{Parser, Subcommand};

#[derive(Clone, clap::ValueEnum)]
pub enum ExportFormat {
    Script,
    Markdown,
}

#[derive(Parser)]
#[command(name = "pepys")]
#[command(version, about = "A command history tool that records shell commands with metadata", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a command execution to history
    Add {
        /// The command that was executed
        #[arg(short, long)]
        command: String,

        /// Exit code of the command
        #[arg(short, long)]
        exit_code: i32,

        /// Duration in milliseconds
        #[arg(short, long)]
        duration_ms: i64,

        /// Working directory where the command was executed
        #[arg(short, long)]
        working_directory: Option<String>,

        /// Output of the command (optional)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Toggle command output recording on/off
    Record,

    /// Browse command history in an interactive TUI
    Browse {
        /// Number of recent commands to load initially
        #[arg(short, long, default_value = "1000")]
        limit: usize,
    },

    /// List recent commands
    List {
        /// Number of commands to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Add or update annotation for a command
    Annotate {
        /// ID of the command to annotate
        id: i64,

        /// Annotation text
        annotation: String,
    },

    /// Export commands to a file
    Export {
        /// IDs of commands to export
        #[arg(required = true)]
        ids: Vec<i64>,

        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Export format
        #[arg(short, long, value_enum, default_value = "script")]
        format: ExportFormat,
    },

    /// Show statistics about command history
    Stats,

    /// Initialize shell integration (prints shell script to stdout)
    Init,

    /// Get a configuration variable value
    Variable {
        /// Name of the variable to get
        name: String,
    },
}
