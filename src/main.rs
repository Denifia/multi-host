#![windows_subsystem = "windows"]

use std::fmt;
use iced::{Element, Subscription, Task, Theme};
use screens::home::HomeScreen;
use screens::settings::SettingsScreen;

mod screens;
mod hosted_process;

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
    StartStopProcess(),
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
            Message::StartStopProcess() |
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
        let subs = self.home_screen.subscription();
        Subscription::batch(subs)
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
