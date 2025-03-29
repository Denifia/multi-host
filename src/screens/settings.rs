use iced::widget::{button, container, row, column, text, text_input};
use iced::{Element, Task};
use iced::Length::Fill;
use crate::{Message, Screen};

#[derive(Debug)]
pub struct SettingsScreen {
    setting_one: String,
    is_dirty: bool,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self { 
            is_dirty: false,
            setting_one: "starting setting".to_owned(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SaveSettings => {
                // todo - save the setting to disk
                self.is_dirty = false;

                Task::none()
            },
            Message::SettingsSettingOneUpdated(value) => {
                self.is_dirty = true;
                self.setting_one = value;

                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let back_button = button("Back")
            .style(button::secondary)
            .on_press(Message::ChangeScreen(Screen::Home));

        let on_press = if self.is_dirty {
            Some(Message::SaveSettings)
        } else {
            None
        };

        let save_button = button("Save")
            .style(button::primary)
            .on_press_maybe(on_press);
        
        let buttons = row![back_button, save_button]
            .spacing(50);
        
        let setting1_input = text_input("", &self.setting_one)
            .on_input(Message::SettingsSettingOneUpdated);
        let setting1 = row![text("Setting One"), setting1_input]
            .spacing(50);
        
        let columns = column![buttons, setting1]
            .padding(50)
            .spacing(50);

        let container = container(columns)
            .width(Fill)
            .height(Fill);

        container.into()
    }
}