use std::process::Command;

use super::task::{BackgroundTask, TaskPoll};

const MAX_HISTORY_LINES: usize = 500;

#[derive(Clone, Copy)]
pub enum EntryKind {
    Command,
    Output,
    Error,
    Info,
}

pub struct TerminalEntry {
    pub text: String,
    pub kind: EntryKind,
}

struct CommandResult {
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
}

pub struct TerminalService {
    history: Vec<TerminalEntry>,
    task: BackgroundTask<CommandResult>,
}

impl Default for TerminalService {
    fn default() -> Self {
        Self {
            history: vec![TerminalEntry {
                text: "Type a command and press Enter.".to_owned(),
                kind: EntryKind::Info,
            }],
            task: BackgroundTask::default(),
        }
    }
}

impl TerminalService {
    pub fn history(&self) -> &[TerminalEntry] {
        &self.history
    }

    pub fn is_running(&self) -> bool {
        self.task.is_running()
    }

    pub fn poll(&mut self) {
        match self.task.poll() {
            TaskPoll::Ready(result) => {
                self.push_lines(&result.stdout, EntryKind::Output);
                self.push_lines(&result.stderr, EntryKind::Error);

                if let Some(exit_code) = result.exit_code.filter(|exit_code| *exit_code != 0) {
                    self.history.push(TerminalEntry {
                        text: format!("Command exited with code {exit_code}."),
                        kind: EntryKind::Error,
                    });
                }

                self.finish_command();
            }
            TaskPoll::Disconnected => {
                self.history.push(TerminalEntry {
                    text: "The command process ended unexpectedly.".to_owned(),
                    kind: EntryKind::Error,
                });
                self.finish_command();
            }
            TaskPoll::Pending => {}
        }
    }

    pub fn submit(&mut self, command: String) {
        let command = command.trim().to_owned();

        if command.is_empty() || self.is_running() {
            return;
        }

        self.history.push(TerminalEntry {
            text: format!("$ {command}"),
            kind: EntryKind::Command,
        });

        if command == "clear" {
            self.history.clear();
            return;
        }

        self.task.start(
            move || match Command::new("sh").args(["-lc", &command]).output() {
                Ok(output) => CommandResult {
                    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                    exit_code: output.status.code(),
                },
                Err(error) => CommandResult {
                    stdout: String::new(),
                    stderr: format!("Could not start the shell: {error}"),
                    exit_code: None,
                },
            },
        );
    }

    fn finish_command(&mut self) {
        self.trim_history();
    }

    fn push_lines(&mut self, text: &str, kind: EntryKind) {
        self.history.extend(text.lines().map(|line| TerminalEntry {
            text: line.to_owned(),
            kind,
        }));
    }

    fn trim_history(&mut self) {
        if self.history.len() > MAX_HISTORY_LINES {
            self.history.drain(..self.history.len() - MAX_HISTORY_LINES);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_empty_commands() {
        let mut terminal = TerminalService::default();
        terminal.submit("   ".to_owned());

        assert_eq!(terminal.history().len(), 1);
        assert!(!terminal.is_running());
    }

    #[test]
    fn clear_removes_history() {
        let mut terminal = TerminalService::default();
        terminal.submit("clear".to_owned());

        assert!(terminal.history().is_empty());
        assert!(!terminal.is_running());
    }
}
