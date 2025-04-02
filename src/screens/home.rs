use crate::hosted_process::ProcessStatus;
use crate::{Message, Screen, hosted_process::HostedProcess};
use iced::Length::{Fill, FillPortion};
use iced::futures::channel::mpsc::Sender;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Subscription, Task};
use std::fmt::Write;

#[derive(Debug)]
pub struct HomeScreen {
    pub hosted_processes: Vec<HostedProcess>,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self {
            hosted_processes: vec![HostedProcess::new("process one".to_owned())],
        }
    }

    pub fn start_stop(&mut self, sender: &Sender<Message>, id: usize) -> Task<Message> {
        let process = &mut self.hosted_processes[id];

        match process.status {
            ProcessStatus::NotRun | ProcessStatus::Stopped => match process.start(sender.clone()) {
                Ok(_) => process.run(),
                Err(_) => writeln!(process.output, "error starting process").unwrap(),
            },
            ProcessStatus::Running => {
                process.stop();
            }
        };

        Task::none()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ProcessOutput(line) => {
                writeln!(self.hosted_processes[0].output, "{}", line)
                    .expect("appending output failed");

                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn subscription(&self) -> Vec<Subscription<Message>> {
        vec![Subscription::run(
            HostedProcess::subscribe_to_process_outputs,
        )]
    }

    pub fn view(&self) -> Element<Message> {
        let settings_button = button("Settings").on_press(Message::ChangeScreen(Screen::Settings));
        let top_pane = container(settings_button)
            .width(Fill)
            .style(container::rounded_box)
            .padding(10);

        let right_pane_text = text(self.hosted_processes[0].output.clone());
        let right_pane = container(right_pane_text)
            .width(FillPortion(4))
            .height(Fill)
            .padding(10);

        let processes: Vec<iced::Element<Message>> = self
            .hosted_processes
            .iter()
            .enumerate()
            .map(|(i, process)| {
                button(process.display_name.as_str())
                    .style(button::primary)
                    .width(Fill)
                    .on_press(Message::StartStopProcess(i))
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

        let all_panes = column![top_pane, middle_pane, bottom_pane].spacing(3);

        let main_window = container(all_panes)
            .center_x(Fill)
            .center_y(Fill)
            .width(Fill)
            .height(Fill);

        container(main_window).into()
    }
}
