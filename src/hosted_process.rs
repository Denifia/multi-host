use iced::futures::executor::block_on;
use iced::futures::{SinkExt, Stream};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::{env, fmt, thread};

use crate::Message;

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

    pub fn start_process() -> impl Stream<Item = Message> {
        iced::stream::channel(100, |mut output| async move {
            println!("start_process entered");
            let channel_error = "channel send failed";

            // todo - the process path with come from config
            let process_path = env::current_dir().and_then(|mut dir| {
                dir.push("example-process");
                Ok(dir)
            });

            if process_path.is_err() {
                // todo - I'd love to have this control flow in '.or_else' but
                // that method doesn't appear to work with async
                output
                    .send(Message::ProcessOutput(format!(
                        "process path error: {:?}",
                        process_path.unwrap_err()
                    )))
                    .await
                    .expect(channel_error);
                return;
            }

            let mut cmd = Command::new("cargo");
            cmd.args(["run", "-q", "--", "--forever"])
                .current_dir(&process_path.unwrap());

            // todo - make sure the child process is correctly cleanup up on multi-host exit
            //cmd.kill_on_drop(true);

            output
                .send(Message::ProcessOutput(
                    "Spawning child process..".to_owned(),
                ))
                .await
                .expect(channel_error);

            // Make sure the child process get's it's own pipes for stdio. If we don't
            // do this, the child processes io is piped to the parents - we don't want that.
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
            cmd.stdin(Stdio::piped());

            let mut child = cmd.spawn().expect("child process did not spawn");

            let stdout = child
                .stdout
                .take()
                .expect("child process did not have stdout");

            // todo - read from the stderr too

            // todo - do I need a Thread to wait for the process to end?
            // can't I just have a periodic Task check for the exit code?
            let mut exit_output = output.clone();
            thread::spawn(move || {
                block_on(async {
                    let status = child.wait().expect("child process encountered an error");
                    exit_output
                        .send(Message::ProcessOutput(format!(
                            "process exited with code {}",
                            status
                        )))
                        .await
                        .expect(channel_error);
                })
            });

            // todo - again this should likely be a Task, not a Thread
            let mut reader = BufReader::new(stdout).lines();
            let mut stdout_output = output.clone();
            thread::spawn(move || {
                block_on(async {
                    // todo - this just throws when the stdout ends. Find a better way.
                    while let Ok(line) = reader.next().expect("reading stdout failed") {
                        stdout_output
                            .send(Message::ProcessOutput(line))
                            .await
                            .expect(channel_error)
                    }
                })
            });
        })
    }
}
