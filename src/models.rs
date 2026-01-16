use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
}

impl CommandRecord {
    pub fn new(
        command: String,
        exit_code: i32,
        duration_ms: i64,
        working_directory: String,
        output: Option<String>,
    ) -> Self {
        Self {
            id: None,
            command,
            exit_code,
            duration_ms,
            timestamp: Utc::now(),
            working_directory,
            output,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_formatting() {
        let cmd = CommandRecord::new("test".to_string(), 0, 500, "/tmp".to_string(), None);
        assert_eq!(cmd.duration_human_readable(), "500ms");

        let cmd = CommandRecord::new("test".to_string(), 0, 1500, "/tmp".to_string(), None);
        assert_eq!(cmd.duration_human_readable(), "1.50s");

        let cmd = CommandRecord::new("test".to_string(), 0, 65000, "/tmp".to_string(), None);
        assert_eq!(cmd.duration_human_readable(), "1m 5s");

        let cmd = CommandRecord::new("test".to_string(), 0, 3665000, "/tmp".to_string(), None);
        assert_eq!(cmd.duration_human_readable(), "1h 1m");
    }

    #[test]
    fn test_status_symbol() {
        let cmd = CommandRecord::new("test".to_string(), 0, 100, "/tmp".to_string(), None);
        assert_eq!(cmd.status_symbol(), "✓");

        let cmd = CommandRecord::new("test".to_string(), 1, 100, "/tmp".to_string(), None);
        assert_eq!(cmd.status_symbol(), "✗");
    }
}
