use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

pub enum TaskPoll<T> {
    Pending,
    Ready(T),
    Disconnected,
}

pub struct BackgroundTask<T> {
    receiver: Option<Receiver<T>>,
}

impl<T> Default for BackgroundTask<T> {
    fn default() -> Self {
        Self { receiver: None }
    }
}

impl<T> BackgroundTask<T>
where
    T: Send + 'static,
{
    pub fn is_running(&self) -> bool {
        self.receiver.is_some()
    }

    pub fn start<F>(&mut self, operation: F) -> bool
    where
        F: FnOnce() -> T + Send + 'static,
    {
        if self.is_running() {
            return false;
        }

        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        thread::spawn(move || {
            let _ = sender.send(operation());
        });
        true
    }

    pub fn poll(&mut self) -> TaskPoll<T> {
        let result = self.receiver.as_ref().map(Receiver::try_recv);
        match result {
            Some(Ok(value)) => {
                self.receiver = None;
                TaskPoll::Ready(value)
            }
            Some(Err(TryRecvError::Disconnected)) => {
                self.receiver = None;
                TaskPoll::Disconnected
            }
            Some(Err(TryRecvError::Empty)) | None => TaskPoll::Pending,
        }
    }

    pub fn cancel(&mut self) {
        self.receiver = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn runs_one_operation_and_returns_its_result() {
        let (release_sender, release_receiver) = mpsc::channel();
        let mut task = BackgroundTask::default();

        assert!(task.start(move || {
            release_receiver.recv().expect("release signal");
            42
        }));
        assert!(!task.start(|| 7));
        assert!(task.is_running());
        release_sender.send(()).expect("send release signal");

        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            match task.poll() {
                TaskPoll::Ready(value) => {
                    assert_eq!(value, 42);
                    break;
                }
                TaskPoll::Pending if Instant::now() < deadline => std::thread::yield_now(),
                TaskPoll::Pending => panic!("background operation did not finish"),
                TaskPoll::Disconnected => panic!("background operation disconnected"),
            }
        }

        assert!(!task.is_running());
    }
}
