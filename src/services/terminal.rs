use std::{
    process::Command,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

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
    result_receiver: Option<Receiver<CommandResult>>,
    running: bool,
}

impl Default for TerminalService {
    fn default() -> Self {
        Self {
            history: vec![TerminalEntry {
                text: "Type a command and press Enter.".to_owned(),
                kind: EntryKind::Info,
            }],
            result_receiver: None,
            running: false,
        }
    }
}

impl TerminalService {
    pub fn history(&self) -> &[TerminalEntry] {
        &self.history
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn poll(&mut self) {
        let result = self.result_receiver.as_ref().map(Receiver::try_recv);

        match result {
            Some(Ok(result)) => {
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
            Some(Err(TryRecvError::Disconnected)) => {
                self.history.push(TerminalEntry {
                    text: "The command process ended unexpectedly.".to_owned(),
                    kind: EntryKind::Error,
                });
                self.finish_command();
            }
            Some(Err(TryRecvError::Empty)) | None => {}
        }
    }

    pub fn submit(&mut self, command: String) {
        let command = command.trim().to_owned();

        if command.is_empty() || self.running {
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

        let (sender, receiver) = mpsc::channel();
        self.result_receiver = Some(receiver);
        self.running = true;

        thread::spawn(move || {
            let result = match Command::new("sh").args(["-lc", &command]).output() {
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
            };

            let _ = sender.send(result);
        });
    }

    fn finish_command(&mut self) {
        self.result_receiver = None;
        self.running = false;
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
