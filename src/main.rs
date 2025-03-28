use iced::{Element, Task, Theme};
use screens::home::HomeScreen;
use screens::settings::SettingsScreen;

mod screens;
fn main() -> iced::Result {
    iced::application("Multi-Host", MultiHost::update, MultiHost::view)
        .theme(MultiHost::theme)
        .run_with(|| (MultiHost::new(), iced::Task::none()))
}

#[derive(Debug)]
struct MultiHost {
    current_screen: Screen,
    home_screen: HomeScreen,
    settings_screen: SettingsScreen
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
    SettingsSettingOneUpdated(String)
}

impl MultiHost {
    fn new() -> Self {
        Self { 
            current_screen: Screen::Home,
            home_screen: HomeScreen::new(),
            settings_screen: SettingsScreen::new(),
        }
    }

    // fn subscribe() -> Task<Message> {
    // }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message  {
            Message::ChangeScreen(screen) => {
                self.current_screen = screen;

                Task::none()
            }
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
