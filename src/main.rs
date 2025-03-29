#![windows_subsystem = "windows"]

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
        output.send(Message::ProcessOutput("first".to_owned())).await.expect("bad1");
        output.send(Message::ProcessOutput("second".to_owned())).await.expect("bad2");
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

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(start_process)
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

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
