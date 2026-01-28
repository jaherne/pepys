use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single command execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecord {
    pub id: Option<i64>,
    pub command: String,
    pub exit_code: i32,
    pub duration_ms: i64,
    pub timestamp: DateTime<Utc>,
    pub working_directory: String,
    pub output: Option<String>,
    pub env_vars: HashMap<String, String>,
}

impl CommandRecord {
    pub fn new(
        command: String,
        exit_code: i32,
        duration_ms: i64,
        working_directory: String,
        output: Option<String>,
        env_vars: HashMap<String, String>,
    ) -> Self {
        Self {
            id: None,
            command,
            exit_code,
            duration_ms,
            timestamp: Utc::now(),
            working_directory,
            output,
            env_vars,
        }
    }

    pub fn duration_human_readable(&self) -> String {
        let ms = self.duration_ms;
        if ms < 1000 {
            format!("{}ms", ms)
        } else if ms < 60000 {
            format!("{:.2}s", ms as f64 / 1000.0)
        } else if ms < 3600000 {
            let minutes = ms / 60000;
            let seconds = (ms % 60000) / 1000;
            format!("{}m {}s", minutes, seconds)
        } else {
            let hours = ms / 3600000;
            let minutes = (ms % 3600000) / 60000;
            format!("{}h {}m", hours, minutes)
        }
    }

    pub fn status_symbol(&self) -> &str {
        if self.exit_code == 0 {
            "✓"
        } else {
            "✗"
        }
    }

    /// Returns a formatted string of environment variables for display,
    /// or None if no environment variables were captured.
    pub fn env_vars_display(&self) -> Option<String> {
        if self.env_vars.is_empty() {
            None
        } else {
            let mut pairs: Vec<_> = self.env_vars.iter().collect();
            pairs.sort_by_key(|(k, _)| *k);
            Some(
                pairs
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_formatting() {
        let cmd = CommandRecord::new("test".to_string(), 0, 500, "/tmp".to_string(), None, HashMap::new());
        assert_eq!(cmd.duration_human_readable(), "500ms");

        let cmd = CommandRecord::new("test".to_string(), 0, 1500, "/tmp".to_string(), None, HashMap::new());
        assert_eq!(cmd.duration_human_readable(), "1.50s");

        let cmd = CommandRecord::new("test".to_string(), 0, 65000, "/tmp".to_string(), None, HashMap::new());
        assert_eq!(cmd.duration_human_readable(), "1m 5s");

        let cmd = CommandRecord::new("test".to_string(), 0, 3665000, "/tmp".to_string(), None, HashMap::new());
        assert_eq!(cmd.duration_human_readable(), "1h 1m");
    }

    #[test]
    fn test_status_symbol() {
        let cmd = CommandRecord::new("test".to_string(), 0, 100, "/tmp".to_string(), None, HashMap::new());
        assert_eq!(cmd.status_symbol(), "✓");

        let cmd = CommandRecord::new("test".to_string(), 1, 100, "/tmp".to_string(), None, HashMap::new());
        assert_eq!(cmd.status_symbol(), "✗");
    }

    #[test]
    fn test_env_vars_display() {
        let cmd = CommandRecord::new("test".to_string(), 0, 100, "/tmp".to_string(), None, HashMap::new());
        assert_eq!(cmd.env_vars_display(), None);

        let mut env_vars = HashMap::new();
        env_vars.insert("NODE_ENV".to_string(), "development".to_string());
        let cmd = CommandRecord::new("test".to_string(), 0, 100, "/tmp".to_string(), None, env_vars);
        assert_eq!(cmd.env_vars_display(), Some("NODE_ENV=development".to_string()));

        let mut env_vars = HashMap::new();
        env_vars.insert("B".to_string(), "2".to_string());
        env_vars.insert("A".to_string(), "1".to_string());
        let cmd = CommandRecord::new("test".to_string(), 0, 100, "/tmp".to_string(), None, env_vars);
        assert_eq!(cmd.env_vars_display(), Some("A=1, B=2".to_string()));
    }
}
