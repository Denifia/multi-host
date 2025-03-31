use iced::futures::channel::mpsc::{self, Sender};
use iced::futures::executor::block_on;
use iced::futures::{SinkExt, Stream, StreamExt};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{env, fmt, thread};

use crate::{Message, MultiHostError};

#[derive(Debug)]
pub struct HostedProcess {
    pub name: String,
    pub status: ProcessStatus,

    // todo - this should be a constrained buffer of some kind
    pub output: String,
}

#[derive(Debug, PartialEq)]
pub enum ProcessStatus {
    NotRun,
    Running,
    Stopped,
}

impl fmt::Display for HostedProcess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status = match self.status {
            ProcessStatus::NotRun => "not run",
            ProcessStatus::Running => "running",
            ProcessStatus::Stopped => "stopped",
        };

        write!(f, "{} ({})", self.name, status)
    }
}

impl HostedProcess {
    pub fn new(name: String) -> HostedProcess {
        Self {
            name,
            status: ProcessStatus::NotRun,
            output: String::new(),
        }
    }

    pub fn start(&self, sender: Sender<Message>) -> Result<(), MultiHostError> {
        // todo - the process path with come from config
        let process_path = env::current_dir()
            .map(|mut dir| {
                dir.push("example-process");
                dir
            })?;

        let mut cmd = Command::new("cargo");
        cmd.args(["run", "-q", "--", "--forever"])
            .current_dir(process_path);

        // todo - make sure the child process is correctly cleanup up on multi-host exit
        //cmd.kill_on_drop(true);

        // Make sure the child process get's it's own pipes for stdio. If we don't
        // do this, the child processes io is piped to the parents - we don't want that.
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::piped());

        let mut child = cmd.spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or(MultiHostError::Simple("couldn't take stdout".to_string()))?;

        let mut exit_output = sender.clone();

        // Thread to wait on the exit of the child process
        thread::spawn(move || {
            block_on(async {
                let status = child.wait().expect("child process encountered an error");
                exit_output
                    .send(Message::ProcessOutput(format!(
                        "process exited with code {}",
                        status
                    )))
                    .await
                    .unwrap();
            })
        });

        let mut reader: std::io::Lines<BufReader<_>> = BufReader::new(stdout).lines();
        let mut stdout_output = sender.clone();

        // Thread to read the stdout of the child process
        thread::spawn(move || {
            block_on(async {
                loop {
                    match reader.next() {
                        Some(result) => {
                            stdout_output
                                .send(Message::ProcessOutput(
                                    result.unwrap_or_else(|e| e.to_string()),
                                ))
                                .await
                                .unwrap();
                        }
                        None => {
                            thread::sleep(Duration::from_secs(1));
                        }
                    }
                }
            })
        });

        Ok(())
    }

    pub fn subscribe_to_process_outputs() -> impl Stream<Item = Message> {
        iced::stream::channel(100, |mut output| async move {
            let (sender, mut receiver) = mpsc::channel(100);
            output
                .send(Message::ListeningForOutput(sender))
                .await
                .unwrap();
            loop {
                let message = receiver.select_next_some().await;
                output.send(message).await.unwrap();
            }
        })
    }
}
