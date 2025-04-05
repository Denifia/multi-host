//#![windows_subsystem = "windows"]
// Uncomment the above before release. Prevents stupid console window.

use iced::{Element, Subscription, Task, Theme, futures::channel::mpsc::Sender};
use screens::home::HomeScreen;
use screens::settings::SettingsScreen;
use std::io;
use thiserror::Error;

mod hosted_process;
mod screens;

fn main() -> iced::Result {
    iced::application("Multi-Host", MultiHost::update, MultiHost::view)
        .theme(MultiHost::theme)
        .subscription(MultiHost::subscription)
        .run_with(|| (MultiHost::new(), iced::Task::none()))
}

#[derive(Debug)]
struct MultiHost {
    current_screen: Screen,
    home_screen: HomeScreen,
    settings_screen: SettingsScreen,
    output_listener: Option<Sender<Message>>,
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
    ProcessOutput(usize, String),
    StartStopProcess(usize),
    ListeningForOutput(Sender<Message>),
    FocusProcess(usize),
    AutoStartProcesses(Sender<Message>),
}

impl MultiHost {
    fn new() -> Self {
        Self {
            current_screen: Screen::Home,
            home_screen: HomeScreen::new(),
            settings_screen: SettingsScreen::new(),
            output_listener: None,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangeScreen(screen) => {
                self.current_screen = screen;
                Task::none()
            }
            Message::StartStopProcess(process_id) => match &self.output_listener {
                Some(listener) => self.home_screen.start_stop(process_id, listener),
                None => panic!("oh no"),
            },
            Message::AutoStartProcesses(sender) => self.home_screen.auto_start(&sender.clone()),
            Message::FocusProcess(process_id) => self.home_screen.focus(process_id),
            Message::ProcessOutput(_, _) => self.home_screen.update(message),
            Message::SettingsSettingOneUpdated(_) | Message::SaveSettings => {
                self.settings_screen.update(message)
            }
            Message::ListeningForOutput(sender) => {
                println!("listening for output, about to signal auto start");
                let message = Message::AutoStartProcesses(sender.clone());
                self.output_listener = Some(sender);
                Task::done(message)
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self.current_screen {
            Screen::Home => self.home_screen.view(),
            Screen::Settings => self.settings_screen.view(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let subs = self.home_screen.subscription();
        Subscription::batch(subs)
    }

    fn theme(&self) -> Theme {
        Theme::CatppuccinLatte
    }
}

#[derive(Debug, Error)]
enum MultiHostError {
    #[error("Iced error: {0}")]
    Iced(#[from] iced::Error),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Simple error: `{0}`")]
    Simple(String),
}
