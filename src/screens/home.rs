use crate::ProcessDefinition;
use crate::hosted_process::ProcessStatus;
use crate::{Message, Screen, hosted_process::HostedProcess};
use iced::Length::{Fill, FillPortion};
use iced::futures::channel::mpsc::Sender;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Subscription, Task};
use std::fmt::Write;
use std::rc::Rc;

#[derive(Debug)]
pub struct HomeScreen {
    pub hosted_processes: Vec<HostedProcess>,
    pub focused_process: usize,
    show_side_bar: bool,
}

impl HomeScreen {
    pub fn new(processes: Rc<Vec<ProcessDefinition>>) -> Self {
        Self {
            hosted_processes: processes
                .iter()
                .map(|pp| HostedProcess::new(pp.clone()))
                .collect(),
            focused_process: 0,
            show_side_bar: true,
        }
    }

    pub fn auto_start(&mut self, sender: &Sender<Message>) -> Task<Message> {
        println!("auto starting processes");
        let _ = self
            .hosted_processes
            .iter_mut()
            .enumerate()
            .for_each(|(process_id, process)| process.try_auto_start(process_id, sender.clone()));
        Task::none()
    }

    pub fn start_stop(&mut self, process_id: usize, sender: &Sender<Message>) -> Task<Message> {
        let process = &mut self.hosted_processes[process_id];

        match process.status {
            ProcessStatus::NotRun | ProcessStatus::Stopped => {
                match process.start(process_id, sender.clone()) {
                    Ok(_) => process.run(),
                    Err(_) => writeln!(process.output, "error starting process").unwrap(),
                }
            }
            ProcessStatus::Running => {
                process.stop();
            }
        };

        Task::none()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ProcessOutput(process_id, line) => {
                writeln!(self.hosted_processes[process_id].output, "{}", line)
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
        let sidebar_text = match self.show_side_bar {
            true => "<<",
            false => ">>",
        };
        let sidebar_toggle_button = button(sidebar_text).on_press(Message::ToggleHomeSideBar);
        let top_pane = container(row!(sidebar_toggle_button, settings_button))
            .width(Fill)
            .style(container::rounded_box)
            .padding(10);

        let right_pane_text = text(self.hosted_processes[self.focused_process].output.clone());
        let right_pane = scrollable(container(right_pane_text).width(FillPortion(4)).padding(10))
            .height(Fill)
            .anchor_bottom();

        // TODO: only do this work if sidebar is enabled
        let processes: Vec<iced::Element<Message>> = self
            .hosted_processes
            .iter()
            .enumerate()
            .map(|(process_id, process)| {
                process.to_element(process_id, process_id == self.focused_process)
            })
            .collect();
        let process_list = iced::widget::Column::with_children(processes);
        let left_pane = scrollable(
            container(process_list)
                .width(FillPortion(1))
                .style(container::rounded_box)
                .padding(0),
        );

        let mut middle_pane = row!().width(Fill).height(Fill).spacing(10);
        middle_pane = match self.show_side_bar {
            true => middle_pane.push(left_pane),
            false => middle_pane,
        };
        middle_pane = middle_pane.push(right_pane);

        let bottom_pane_text = text("bottom pane");
        let bottom_pane = container(bottom_pane_text)
            .width(Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    r: 0.49,
                    g: 0.27,
                    b: 0.62,
                    a: 1.00,
                })),
                ..Default::default()
            })
            .padding(10);

        let all_panes = column![top_pane, middle_pane, bottom_pane].spacing(3);

        let main_window = container(all_panes)
            .center_x(Fill)
            .center_y(Fill)
            .width(Fill)
            .height(Fill);

        container(main_window).into()
    }

    pub fn focus(&mut self, process_id: usize) -> Task<Message> {
        self.focused_process = process_id;
        Task::none()
    }

    pub fn toggle_side_bar(&mut self) -> Task<Message> {
        self.show_side_bar = !self.show_side_bar;
        Task::none()
    }
}
