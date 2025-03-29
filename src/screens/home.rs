use iced::widget::{button, column, container, row, text};
use iced::{Element, Task};
use iced::Length::{Fill, FillPortion};
use crate::{Message, Screen};

#[derive(Debug)]
pub struct HomeScreen {
    hosted_processes: Vec<HostedProcess>,
}

#[derive(Debug)]
struct HostedProcess {
    name: String,
    output: String,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self {
            hosted_processes: vec![
                HostedProcess {
                    name: "Process 1".to_owned(),
                    output: String::new(),
                }
            ],
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ProcessOutput(output) => {
                self.hosted_processes[0].output = format!("{}\n{}", self.hosted_processes[0].output, output);
                
                Task::none()
            },
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let settings_button = button("Settings")
            .on_press(Message::ChangeScreen(Screen::Settings));
        let top_pane = container(settings_button)
            .width(Fill)
            .style(container::rounded_box)
            .padding(10);

        let right_pane_text = text(self.hosted_processes[0].output.clone());
        let right_pane = container(right_pane_text)
            .width(FillPortion(4))
            .height(Fill)
            .padding(10);

        let processes: Vec<iced::Element<Message>>  = self.hosted_processes
            .iter()
            .map(|process| {
                button(process.name.as_str())
                .style(button::secondary)
                .width(Fill)
                .into()
            })
            .collect();
        let process_list = iced::widget::Column::with_children(processes);
        let left_pane = container(process_list)
            .width(FillPortion(1))
            .height(Fill)
            .style(container::rounded_box)
            .padding(10);

        let middle_pane = row!(left_pane, right_pane)
            .width(Fill)
            .height(Fill)
            .spacing(10);

        let bottom_pane_text = text("bottom pane");
        let bottom_pane = container(bottom_pane_text)
            .width(Fill)
            .style(container::rounded_box)
            .padding(10);

        let all_panes = column![top_pane, middle_pane, bottom_pane]
            .spacing(3);

        let main_window = container(all_panes)
            .center_x(Fill)
            .center_y(Fill)
            .width(Fill)
            .height(Fill);

        container(main_window)
            .into()
    }
}