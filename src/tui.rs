use crate::models::CommandRecord;
use crate::storage::Storage;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

pub struct App {
    storage: Storage,
    commands: Vec<CommandRecord>,
    list_state: ListState,
    selected_ids: Vec<i64>,
    show_help: bool,
    show_delete_confirm: bool,
    show_annotate: bool,
    annotation_input: String,
    annotating_idx: Option<usize>,
}

impl App {
    pub fn new(storage: Storage, limit: usize) -> Result<Self> {
        let commands = storage.get_recent(limit)?;
        let mut list_state = ListState::default();
        if !commands.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            storage,
            commands,
            list_state,
            selected_ids: Vec::new(),
            show_help: false,
            show_delete_confirm: false,
            show_annotate: false,
            annotation_input: String::new(),
            annotating_idx: None,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if self.show_annotate {
                    // Handle annotation input
                    match key.code {
                        KeyCode::Enter => {
                            self.save_annotation()?;
                        }
                        KeyCode::Esc => {
                            self.show_annotate = false;
                            self.annotation_input.clear();
                            self.annotating_idx = None;
                        }
                        KeyCode::Backspace => {
                            self.annotation_input.pop();
                        }
                        KeyCode::Char(c) => {
                            self.annotation_input.push(c);
                        }
                        _ => {}
                    }
                } else if self.show_delete_confirm {
                    // Handle delete confirmation
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            self.show_delete_confirm = false;
                            self.delete_confirmed()?;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            self.show_delete_confirm = false;
                        }
                        _ => {}
                    }
                } else {
                    // Normal key handling
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('?') => self.show_help = !self.show_help,
                        KeyCode::Down | KeyCode::Char('j') => self.next(),
                        KeyCode::Up | KeyCode::Char('k') => self.previous(),
                        KeyCode::Char(' ') => self.toggle_selection(),
                        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.select_all()
                        }
                        KeyCode::Char('a') => self.annotate_selected()?,
                        KeyCode::Char('d') => self.show_delete_confirm = true,
                        // Because we're nice we'll handle ctrl-c too
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(())
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(10)])
            .split(f.area());

        self.render_command_list(f, chunks[0]);
        self.render_details(f, chunks[1]);

        // Render overlays on top
        if self.show_help {
            self.render_help(f);
        } else if self.show_delete_confirm {
            self.render_delete_confirm(f);
        } else if self.show_annotate {
            self.render_annotate(f);
        }
    }

    fn render_command_list(&mut self, f: &mut Frame, area: Rect) {
        let current_selection = self.list_state.selected();

        let items: Vec<ListItem> = self
            .commands
            .iter()
            .enumerate()
            .map(|(idx, cmd)| {
                let id = cmd.id.unwrap_or(0);
                let is_selected = self.selected_ids.contains(&id);
                let is_highlighted = current_selection == Some(idx);

                // Choose indicator based on selection and highlight state
                let (indicator, indicator_style) = if is_selected {
                    ("●", Style::default().fg(Color::Cyan))
                } else if is_highlighted {
                    ("○", Style::default().fg(Color::Cyan))
                } else {
                    (" ", Style::default())
                };

                let content = Line::from(vec![
                    Span::styled(indicator, indicator_style),
                    Span::raw(" "),
                    Span::styled(
                        cmd.status_symbol(),
                        Style::default().fg(if cmd.exit_code == 0 {
                            Color::Green
                        } else {
                            Color::Red
                        }),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        &cmd.command,
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" ({})", cmd.duration_human_readable())),
                ]);

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command History (↑↓:navigate Space:select a:annotate d:delete ?:help q:quit)"),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_details(&self, f: &mut Frame, area: Rect) {
        let selected = self.list_state.selected();

        let text = if let Some(idx) = selected {
            if let Some(cmd) = self.commands.get(idx) {
                let mut lines = vec![
                    Line::from(vec![
                        Span::styled("Command: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(&cmd.command),
                    ]),
                    Line::from(vec![
                        Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(
                            format!("{} (exit code: {})", cmd.status_symbol(), cmd.exit_code),
                            Style::default().fg(if cmd.exit_code == 0 {
                                Color::Green
                            } else {
                                Color::Red
                            }),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("Duration: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(cmd.duration_human_readable()),
                    ]),
                    Line::from(vec![
                        Span::styled("Time: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(cmd.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
                    ]),
                    Line::from(vec![
                        Span::styled("Directory: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(&cmd.working_directory),
                    ]),
                ];

                if let Ok(Some(annotation)) = self.storage.get_annotation_for_command(&cmd.command) {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![Span::styled(
                        "Note:",
                        Style::default().add_modifier(Modifier::BOLD),
                    )]));
                    lines.push(Line::from(annotation));
                }

                Text::from(lines)
            } else {
                Text::from("No command selected")
            }
        } else {
            Text::from("No command selected")
        };

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_help(&self, f: &mut Frame) {
        let help_text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Pepys - Command History Browser",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Keyboard Shortcuts:"),
            Line::from("  ↑/k       - Move up"),
            Line::from("  ↓/j       - Move down"),
            Line::from("  Space     - Toggle selection"),
            Line::from("  Ctrl-A    - Select all commands"),
            Line::from("  a         - Annotate selected command"),
            Line::from("  d         - Delete selected command"),
            Line::from("  ?         - Toggle this help"),
            Line::from("  q/Ctrl-C  - Quit"),
            Line::from(""),
            Line::from("Press ? to close this help"),
        ]);

        let paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .style(Style::default().bg(Color::Black)),
            )
            .style(Style::default().bg(Color::Black))
            .wrap(Wrap { trim: true });

        // Help dialog needs: 14 lines of text + 2 for borders = 16 lines minimum
        let area = centered_rect_fixed_height(60, 16, f.area());
        f.render_widget(Clear, area); // Clear the background
        f.render_widget(paragraph, area);
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.commands.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.commands.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn toggle_selection(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if let Some(cmd) = self.commands.get(idx) {
                let id = cmd.id.unwrap_or(0);
                if let Some(pos) = self.selected_ids.iter().position(|&x| x == id) {
                    self.selected_ids.remove(pos);
                } else {
                    self.selected_ids.push(id);
                }
            }
        }
    }

    fn select_all(&mut self) {
        // Select all commands
        self.selected_ids.clear();
        for cmd in &self.commands {
            if let Some(id) = cmd.id {
                self.selected_ids.push(id);
            }
        }
    }

    fn annotate_selected(&mut self) -> Result<()> {
        if let Some(idx) = self.list_state.selected() {
            if let Some(cmd) = self.commands.get(idx) {
                // Pre-fill with existing annotation if present
                self.annotation_input = self
                    .storage
                    .get_annotation_for_command(&cmd.command)?
                    .unwrap_or_default();
                self.annotating_idx = Some(idx);
                self.show_annotate = true;
            }
        }
        Ok(())
    }

    fn save_annotation(&mut self) -> Result<()> {
        if let Some(idx) = self.annotating_idx {
            if let Some(cmd) = self.commands.get(idx) {
                let annotation = if self.annotation_input.is_empty() {
                    None
                } else {
                    Some(self.annotation_input.clone())
                };
                self.storage
                    .set_annotation_for_command(&cmd.command, annotation)?;
            }
        }
        self.show_annotate = false;
        self.annotation_input.clear();
        self.annotating_idx = None;
        Ok(())
    }

    fn render_annotate(&self, f: &mut Frame) {
        let command_preview = if let Some(idx) = self.annotating_idx {
            if let Some(cmd) = self.commands.get(idx) {
                if cmd.command.len() > 50 {
                    format!("{}...", &cmd.command[..47])
                } else {
                    cmd.command.clone()
                }
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };

        let text = Text::from(vec![
            Line::from(vec![
                Span::styled("Command: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(command_preview),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw(&self.annotation_input),
                Span::styled("█", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" = Save   "),
                Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" = Cancel"),
            ]),
        ]);

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title("Add Annotation")
                    .style(Style::default().bg(Color::Black)),
            )
            .style(Style::default().bg(Color::Black))
            .wrap(Wrap { trim: true });

        // Dialog needs: 5 lines of text + 2 for borders = 7 lines minimum
        let area = centered_rect_fixed_height(70, 7, f.area());
        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    fn render_delete_confirm(&self, f: &mut Frame) {
        let delete_count = if self.selected_ids.is_empty() {
            1
        } else {
            self.selected_ids.len()
        };

        let message = if delete_count == 1 {
            let selected = self.list_state.selected();
            let command_text = if let Some(idx) = selected {
                if let Some(cmd) = self.commands.get(idx) {
                    // Truncate long commands to fit better
                    if cmd.command.len() > 60 {
                        format!("{}...", &cmd.command[..57])
                    } else {
                        cmd.command.clone()
                    }
                } else {
                    "unknown command".to_string()
                }
            } else {
                "unknown command".to_string()
            };
            format!("Delete this command?\n\n{}", command_text)
        } else {
            format!("Delete {} selected commands?", delete_count)
        };

        let text = Text::from(vec![
            Line::from(Span::styled(
                message.lines().next().unwrap_or("Delete?"),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Red),
            )),
            Line::from(""),
            Line::from(Span::styled(
                message.lines().skip(2).next().unwrap_or(""),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("y", Style::default().add_modifier(Modifier::BOLD).fg(Color::Green)),
                Span::raw(" = Yes   "),
                Span::styled("n", Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)),
                Span::raw(" = No   "),
                Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" = Cancel"),
            ]),
        ]);

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .title("Confirm Delete")
                    .style(Style::default().bg(Color::Black)),
            )
            .style(Style::default().bg(Color::Black))
            .wrap(Wrap { trim: true });

        // Dialog needs: 5 lines of text + 2 for borders + 1 for title = 8 lines minimum
        let area = centered_rect_fixed_height(70, 8, f.area());
        f.render_widget(Clear, area); // Clear the background
        f.render_widget(paragraph, area);
    }

    fn delete_confirmed(&mut self) -> Result<()> {
        if self.selected_ids.is_empty() {
            // No selections - delete current line only
            if let Some(idx) = self.list_state.selected() {
                if let Some(cmd) = self.commands.get(idx) {
                    let id = cmd.id.unwrap_or(0);
                    self.storage.delete(id)?;
                    self.commands.remove(idx);

                    // Adjust selection
                    if self.commands.is_empty() {
                        self.list_state.select(None);
                    } else if idx >= self.commands.len() {
                        self.list_state.select(Some(self.commands.len() - 1));
                    }
                }
            }
        } else {
            // Delete all selected items
            let ids_to_delete = self.selected_ids.clone();

            // Delete from storage
            for id in &ids_to_delete {
                self.storage.delete(*id)?;
            }

            // Remove from commands list (iterate in reverse to maintain indices)
            let mut removed_count = 0;
            self.commands.retain(|cmd| {
                let should_keep = !ids_to_delete.contains(&cmd.id.unwrap_or(0));
                if !should_keep {
                    removed_count += 1;
                }
                should_keep
            });

            // Clear the selection list
            self.selected_ids.clear();

            // Adjust the current selection
            if self.commands.is_empty() {
                self.list_state.select(None);
            } else if let Some(current_idx) = self.list_state.selected() {
                if current_idx >= self.commands.len() {
                    self.list_state.select(Some(self.commands.len() - 1));
                }
            }
        }
        Ok(())
    }
}

fn centered_rect_fixed_height(percent_x: u16, height: u16, r: Rect) -> Rect {
    // Calculate how much height is available
    let available_height = r.height;

    // Use the requested height if it fits, otherwise use percentage
    let actual_height = if height <= available_height {
        height
    } else {
        // Fall back to 80% of available height if content doesn't fit
        (available_height * 80) / 100
    };

    let vertical_margin = (available_height.saturating_sub(actual_height)) / 2;

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_margin),
            Constraint::Length(actual_height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
