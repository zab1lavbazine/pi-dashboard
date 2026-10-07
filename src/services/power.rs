use std::{
    process::Command,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PowerAction {
    Shutdown,
    Reboot,
}

impl PowerAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Shutdown => "Shut down",
            Self::Reboot => "Reboot",
        }
    }

    pub fn confirmation_text(self) -> &'static str {
        match self {
            Self::Shutdown => "Are you sure you want to shut down the dashboard?",
            Self::Reboot => "Are you sure you want to reboot the dashboard?",
        }
    }

    fn command(self) -> &'static str {
        match self {
            Self::Shutdown => "poweroff",
            Self::Reboot => "reboot",
        }
    }
}

#[derive(Default)]
pub struct PowerService {
    confirmation: Option<PowerAction>,
    task_receiver: Option<Receiver<Result<String, String>>>,
    message: Option<(String, bool)>,
}

impl PowerService {
    pub fn confirmation(&self) -> Option<PowerAction> {
        self.confirmation
    }

    pub fn request_confirmation(&mut self, action: PowerAction) {
        if !self.is_busy() {
            self.confirmation = Some(action);
            self.message = None;
        }
    }

    pub fn cancel_confirmation(&mut self) {
        self.confirmation = None;
    }

    pub fn is_busy(&self) -> bool {
        self.task_receiver.is_some()
    }

    pub fn message(&self) -> Option<(&str, bool)> {
        self.message
            .as_ref()
            .map(|(message, is_error)| (message.as_str(), *is_error))
    }

    pub fn confirm(&mut self) {
        if self.is_busy() {
            return;
        }

        let Some(action) = self.confirmation.take() else {
            return;
        };

        let (sender, receiver) = mpsc::channel();
        self.task_receiver = Some(receiver);
        self.message = None;

        thread::spawn(move || {
            let _ = sender.send(run_action(action));
        });
    }

    pub fn poll(&mut self) {
        let result = self.task_receiver.as_ref().map(Receiver::try_recv);

        match result {
            Some(Ok(Ok(message))) => {
                self.message = Some((message, false));
                self.task_receiver = None;
            }
            Some(Ok(Err(error))) => {
                self.message = Some((error, true));
                self.task_receiver = None;
            }
            Some(Err(TryRecvError::Disconnected)) => {
                self.message = Some(("Power command ended unexpectedly.".to_owned(), true));
                self.task_receiver = None;
            }
            Some(Err(TryRecvError::Empty)) | None => {}
        }
    }
}

fn run_action(action: PowerAction) -> Result<String, String> {
    let output = Command::new("systemctl")
        .arg("--no-ask-password")
        .arg(action.command())
        .output()
        .map_err(|error| format!("Could not run systemctl: {error}"))?;

    if output.status.success() {
        Ok(format!("{} command accepted.", action.label()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(if stderr.is_empty() {
            format!("{} command failed.", action.label())
        } else {
            stderr
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_actions_map_to_systemctl_commands() {
        assert_eq!(PowerAction::Shutdown.command(), "poweroff");
        assert_eq!(PowerAction::Reboot.command(), "reboot");
    }

    #[test]
    fn confirmation_can_be_cancelled() {
        let mut service = PowerService::default();
        service.request_confirmation(PowerAction::Shutdown);
        assert_eq!(service.confirmation(), Some(PowerAction::Shutdown));

        service.cancel_confirmation();
        assert_eq!(service.confirmation(), None);
    }
}
