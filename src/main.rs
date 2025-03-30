#![windows_subsystem = "windows"]

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::{env, fmt, thread};
use iced::futures::executor::block_on;
use iced::futures::{SinkExt, Stream};
use iced::{Element, Subscription, Task, Theme};
use screens::home::HomeScreen;
use screens::settings::SettingsScreen;

mod screens;

fn main() -> iced::Result {
    iced::application("Multi-Host", MultiHost::update, MultiHost::view)
        .theme(MultiHost::theme)
        .subscription(MultiHost::subscription)
        .run_with(|| (MultiHost::new(), iced::Task::none()))
}

fn start_process()  -> impl Stream<Item = Message> {
    iced::stream::channel(100, |mut output| async move {
        let channel_error = "channel send failed";

        // todo - the process path with come from config
        let process_path = env::current_dir()
            .and_then(|mut dir| {
                dir.push("example-process");
                Ok(dir)
            });
        
        if process_path.is_err() {
            // todo - I'd love to have this control flow in '.or_else' but 
            // that method doesn't appear to work with async
            output.send(Message::ProcessOutput(format!("process path error: {:?}", process_path.unwrap_err()))).await
                .expect(channel_error);
            return;
        }

        let mut cmd = Command::new("cargo");
        cmd.args(["run", "-q", "--", "--throw"]).current_dir(&process_path.unwrap());

        // todo - make sure the child process is correctly cleanup up on multi-host exit
        //cmd.kill_on_drop(true); 

        output.send(Message::ProcessOutput("Spawning child process..".to_owned())).await
            .expect(channel_error);

        // Make sure the child process get's it's own pipes for stdio. If we don't 
        // do this, the child processes io is piped to the parents - we don't want that.
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::piped());

        let mut child = cmd.spawn()
            .expect("child process did not spawn");

        let stdout = child.stdout
            .take()
            .expect("child process did not have stdout");

        // todo - read from the stderr too

        // todo - do I need a Thread to wait for the process to end?
        // can't I just have a periodic Task check for the exit code?
        let mut exit_output = output.clone();
        thread::spawn(move || block_on(async {
            let status = child.wait()
                .expect("child process encountered an error");
            exit_output.send(Message::ProcessOutput(format!("process exited with code {}", status))).await
                .expect(channel_error);
        }));

        // todo - again this should likely be a Task, not a Thread
        let mut reader = BufReader::new(stdout).lines();
        let mut stdout_output = output.clone();
        thread::spawn(move || block_on(async {
            // todo - this just throws when the stdout ends. Find a better way.
            while let Ok(line) = reader.next().expect("reading stdout failed") {
                stdout_output.send(Message::ProcessOutput(line)).await
                    .expect(channel_error)
            }
        }));
    })
}

#[derive(Debug)]
struct MultiHost {
    current_screen: Screen,
    home_screen: HomeScreen,
    settings_screen: SettingsScreen,
}

#[derive(Debug, PartialEq, Clone)]
enum Screen {
    Home,
    Settings,
}

#[derive(Debug, Clone)]
enum Message {
    ChangeScreen(Screen),
    SaveSettings,
    SettingsSettingOneUpdated(String),
    ProcessOutput(String),
}

impl MultiHost {
    fn new() -> Self {
        Self { 
            current_screen: Screen::Home,
            home_screen: HomeScreen::new(),
            settings_screen: SettingsScreen::new(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message  {
            // Screen navigation
            Message::ChangeScreen(screen) => {
                self.current_screen = screen;

                Task::none()
            }

            // Home Screen
            Message::ProcessOutput(_) => self.home_screen.update(message),

            // Settings Screen
            Message::SettingsSettingOneUpdated(_) |
            Message::SaveSettings => self.settings_screen.update(message),
        }
    }

    fn view(&self) -> Element<Message> {
        match self.current_screen {
            Screen::Home => self.home_screen.view(),
            Screen::Settings => self.settings_screen.view(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(start_process)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

#[derive(Debug)]
enum MultiHostError {
    Iced(iced::Error),
} 

impl From<iced::Error> for MultiHostError {
    fn from(err: iced::Error) -> MultiHostError {
        MultiHostError::Iced(err)
    }
}

impl fmt::Display for MultiHostError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MultiHostError::Iced(ref err) => write!(f, "Iced error: {}", err),
        }
    }
}
