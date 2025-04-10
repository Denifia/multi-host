use iced::Length::Fill;
use iced::futures::channel::mpsc::{self, Sender};
use iced::futures::executor::block_on;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::widget::{button, row};
use std::fmt::Write;
use std::io::{BufRead, BufReader, Lines};
use std::path::PathBuf;
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::{env, fmt, thread};

use crate::{Message, MultiHostError, ProcessDefinition};

#[derive(Debug)]
pub struct HostedProcess {
    pub name: String,
    pub status: ProcessStatus,
    auto_start_enabled: bool,
    // todo - this should be a constrained buffer of some kind
    pub output: String,
    pub display_name: String,
    pub child: Option<Arc<Mutex<Child>>>,
    app: String,
    args: Vec<String>,
    working_directory: PathBuf,
}

#[derive(Debug, PartialEq)]
pub enum ProcessStatus {
    NotRun,
    Running,
    Stopped,
}

impl fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            ProcessStatus::NotRun => "not run",
            ProcessStatus::Running => "running",
            ProcessStatus::Stopped => "stopped",
        };
        write!(f, "{}", status)
    }
}

impl fmt::Display for HostedProcess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display_name)
    }
}

impl HostedProcess {
    pub fn new(config: ProcessDefinition) -> HostedProcess {
        let status = ProcessStatus::NotRun;
        let display_name = HostedProcess::display_name(config.name.as_str(), &status);
        Self {
            name: config.name,
            status,
            output: String::new(),
            display_name,
            child: None,
            auto_start_enabled: config.auto_start,
            app: config.command,
            args: config.args,
            working_directory: PathBuf::from(config.cwd),
        }
    }

    fn update_display_name(&mut self) {
        self.display_name = HostedProcess::display_name(self.name.as_str(), &self.status);
    }

    fn display_name(name: &str, status: &ProcessStatus) -> String {
        format!("{} ({})", name, status)
    }

    pub fn run(&mut self) {
        self.status = ProcessStatus::Running;
        self.update_display_name();
    }

    pub fn stop(&mut self) {
        self.status = ProcessStatus::Stopped;
        self.update_display_name();

        match self.child.as_ref() {
            Some(c) => c
                .lock()
                .unwrap()
                .kill()
                .expect("child process should be killed"),
            None => (),
        }
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

    pub fn try_auto_start(&mut self, process_id: usize, sender: Sender<Message>) {
        match self.auto_start_enabled {
            false => (),
            true => match self.start(process_id, sender) {
                Ok(_) => self.run(),
                Err(e) => writeln!(self.output, "error starting process {:?}", e).unwrap(),
            },
        }
    }

    pub fn start(
        &mut self,
        process_id: usize,
        sender: Sender<Message>,
    ) -> Result<(), MultiHostError> {
        let mut cmd = Command::new(self.app.clone());

        cmd.args(self.args.clone());
        cmd.current_dir(self.working_directory.clone());

        // todo - support light blue
        // let config_path = format!("{}.config", self.app);
        // cmd.env("LightBlueRunMode", "process");
        // cmd.env("LightBlueHost", "true");
        // cmd.env("LightBlueConfigurationPath", config_path);
        // cmd.env("LightBlueRoleName", self.name.clone());
        // cmd.env("LightBlueUseHostedStorage", "true");

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

        let stderr = child
            .stderr
            .take()
            .ok_or(MultiHostError::Simple("couldn't take stderr".to_string()))?;

        let arc_child = Arc::new(Mutex::new(child));

        let exit_child = Arc::clone(&arc_child);
        let mut exit_sender = sender.clone();

        // Thread to wait on the exit of the child process
        thread::spawn(move || {
            block_on(HostedProcess::poll_for_exit_code(
                process_id,
                exit_child,
                &mut exit_sender,
            ));
        });

        // Thread to read the stdout of the child process
        let mut output_sender = sender.clone();
        thread::spawn(move || {
            block_on(HostedProcess::poll_for_std_output(
                process_id,
                &mut BufReader::new(stdout).lines(),
                &mut output_sender,
            ))
        });

        // Thread to read the stderr of the child process
        thread::spawn(move || {
            block_on(HostedProcess::poll_for_std_error(
                process_id,
                &mut BufReader::new(stderr).lines(),
                &mut sender.clone(),
            ))
        });

        self.child = Some(arc_child);

        Ok(())
    }

    pub fn to_element(&self, process_id: usize, is_focused: bool) -> iced::Element<Message> {
        let action_button = match self.status {
            ProcessStatus::NotRun | ProcessStatus::Stopped => button("start")
                .style(button::success)
                .on_press(Message::StartStopProcess(process_id)),
            ProcessStatus::Running => button("stop")
                .style(button::danger)
                .on_press(Message::StartStopProcess(process_id)),
        };
        row![
            match is_focused {
                true => button(self.name.as_str())
                    .style(button::primary)
                    .width(Fill),
                false => button(self.name.as_str())
                    .style(button::primary)
                    .width(Fill)
                    .on_press(Message::FocusProcess(process_id)),
            },
            action_button
        ]
        .into()
    }

    async fn poll_for_std_output(
        process_id: usize,
        stdout_reader: &mut Lines<BufReader<ChildStdout>>,
        output: &mut Sender<Message>,
    ) {
        let _ = output
            .send(Message::ProcessOutput(
                process_id,
                "process starting...".to_string(),
            ))
            .await;

        loop {
            let stdout_ok: bool = match stdout_reader.next() {
                Some(result) => output
                    .send(Message::ProcessOutput(
                        process_id,
                        result.unwrap_or_else(|e| e.to_string()),
                    ))
                    .await
                    .is_ok(),
                _ => true,
            };

            if stdout_ok == false {
                break;
            } else {
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    async fn poll_for_std_error(
        process_id: usize,
        stderr_reader: &mut Lines<BufReader<ChildStderr>>,
        output: &mut Sender<Message>,
    ) {
        loop {
            let stderr_ok: bool = match stderr_reader.next() {
                Some(err) => output
                    .send(Message::ProcessOutput(
                        process_id,
                        err.unwrap_or_else(|e| e.to_string()),
                    ))
                    .await
                    .is_ok(),
                _ => true,
            };

            if stderr_ok == false {
                break;
            } else {
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    async fn poll_for_exit_code(
        process_id: usize,
        child: Arc<Mutex<Child>>,
        output: &mut Sender<Message>,
    ) {
        loop {
            let exit = {
                let mut child = child.lock().unwrap();
                match child.try_wait() {
                    Ok(optional_status) => match optional_status {
                        Some(status) => {
                            output
                                .send(Message::ProcessOutput(
                                    process_id,
                                    format!("process exited with code {}", status),
                                ))
                                .await
                                .unwrap();
                            Some(status)
                        }
                        None => None,
                    },
                    Err(_) => panic!("oh no"),
                }
            };
            match exit {
                Some(_) => break,
                None => thread::sleep(Duration::from_secs(1)),
            }
        }
    }
}
