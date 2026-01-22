mod cli;
mod config;
mod export;
mod models;
mod recorder;
mod storage;
mod tui;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use export::Exporter;
use recorder::{generate_bash_integration, generate_zsh_integration, Recorder};
use storage::Storage;
use std::path::PathBuf;

fn get_db_path() -> Result<PathBuf> {
    let data_dir = get_data_dir()?;
    Ok(data_dir.join("history.db"))
}

fn get_data_dir() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .context("Failed to determine data directory")?
        .join("pepys");

    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("Failed to create data directory: {:?}", data_dir))?;

    Ok(data_dir)
}

fn get_recording_state_path() -> Result<PathBuf> {
    let data_dir = get_data_dir()?;
    Ok(data_dir.join("recording_enabled"))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_path = get_db_path()?;
    let storage = Storage::new(db_path)?;

    match cli.command {
        Commands::Add {
            command,
            exit_code,
            duration_ms,
            working_directory,
            output,
        } => {
            let recorder = Recorder::new(storage);
            recorder.record(command, exit_code, duration_ms, working_directory, output)?;
        }

        Commands::Record => {
            let state_path = get_recording_state_path()?;
            let is_enabled = state_path.exists();

            if is_enabled {
                // Disable recording
                std::fs::remove_file(&state_path)
                    .with_context(|| format!("Failed to remove state file: {:?}", state_path))?;
                println!("Recording disabled");
            } else {
                // Enable recording
                std::fs::write(&state_path, "")
                    .with_context(|| format!("Failed to create state file: {:?}", state_path))?;
                println!("Recording enabled");
            }
        }

        Commands::Browse { limit } => {
            let mut app = tui::App::new(storage, limit)?;
            app.run()?;
        }

        Commands::List { limit } => {
            let commands = storage.get_recent(limit)?;

            println!("\nRecent Commands:");
            println!("{}", "=".repeat(80));

            for cmd in commands {
                println!(
                    "{} [{}] {} - {} ({})",
                    cmd.status_symbol(),
                    cmd.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    cmd.command,
                    cmd.duration_human_readable(),
                    cmd.working_directory
                );
                if let Some(annotation) = storage.get_annotation_for_command(&cmd.command)? {
                    println!("    Note: {}", annotation);
                }
            }
        }

        Commands::Annotate { id, annotation } => {
            if let Some(cmd) = storage.get(id)? {
                storage.set_annotation_for_command(&cmd.command, Some(annotation.clone()))?;
                println!("✓ Added annotation to command: {}", cmd.command);
            } else {
                eprintln!("Command #{} not found", id);
                std::process::exit(1);
            }
        }

        Commands::ExportScript { ids, output } => {
            let mut records = Vec::new();
            for id in ids {
                if let Some(record) = storage.get(id)? {
                    records.push(record);
                } else {
                    eprintln!("Warning: Command #{} not found", id);
                }
            }

            Exporter::export_bash_script(&records, &output, &storage)?;
            println!("✓ Exported {} commands to {}", records.len(), output);
        }

        Commands::ExportMarkdown { ids, output } => {
            let mut records = Vec::new();
            for id in ids {
                if let Some(record) = storage.get(id)? {
                    records.push(record);
                } else {
                    eprintln!("Warning: Command #{} not found", id);
                }
            }

            Exporter::export_markdown(&records, &output, &storage)?;
            println!("✓ Exported {} commands to {}", records.len(), output);
        }

        Commands::Stats => {
            let total = storage.count()?;
            let commands = storage.get_all()?;

            let successful = commands.iter().filter(|c| c.exit_code == 0).count();
            let failed = total - successful;

            let total_duration_ms: i64 = commands.iter().map(|c| c.duration_ms).sum();
            let avg_duration_ms = if total > 0 {
                total_duration_ms / total as i64
            } else {
                0
            };

            println!("\nCommand History Statistics:");
            println!("{}", "=".repeat(80));
            println!("Total commands: {}", total);
            println!("Successful: {} ({:.1}%)", successful, (successful as f64 / total as f64) * 100.0);
            println!("Failed: {} ({:.1}%)", failed, (failed as f64 / total as f64) * 100.0);
            println!("Average duration: {}ms", avg_duration_ms);

            if let Some(longest) = commands.iter().max_by_key(|c| c.duration_ms) {
                println!("\nLongest command:");
                println!("  {}", longest.command);
                println!("  Duration: {}", longest.duration_human_readable());
            }

            if let Some(latest) = commands.first() {
                println!("\nMost recent command:");
                println!("  {}", latest.command);
                println!("  {} - {}", latest.timestamp.format("%Y-%m-%d %H:%M:%S"), latest.status_symbol());
            }
        }

        Commands::Init { shell } => {
            let cfg = config::Config::load()?;
            let script = match shell.as_str() {
                "bash" => generate_bash_integration(&cfg),
                "zsh" => generate_zsh_integration(&cfg),
                _ => {
                    eprintln!("Unsupported shell: {}", shell);
                    eprintln!("Supported shells: bash, zsh");
                    std::process::exit(1);
                }
            };

            println!("{}", script);
            eprintln!("\n# To enable pepys integration, add this to your shell config:");
            eprintln!("# eval \"$(pepys init --shell {})\"", shell);
        }

        Commands::Variable { name, shell } => {
            let cfg = config::Config::load()?;

            match name.as_str() {
                "placeholder_color" => {
                    println!("{}", cfg.placeholder_color);
                }
                "recording_indicator" => {
                    let state_path = get_recording_state_path()?;
                    if state_path.exists() {
                        let shell = shell.unwrap_or_else(|| "bash".to_string());
                        match shell.as_str() {
                            "bash" => {
                                print!("\x1b[{}m[●]\x1b[0m ", cfg.bash_color_code());
                            }
                            "zsh" => {
                                print!("%F{{{}}}[●]%f ", cfg.zsh_color_name());
                            }
                            _ => {
                                eprintln!("Unsupported shell: {}", shell);
                                std::process::exit(1);
                            }
                        }
                    }
                    // If recording is disabled, output nothing
                }
                _ => {
                    eprintln!("Unknown variable: {}", name);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
