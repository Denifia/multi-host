use iced::widget::{button, column, container, row, text};
use iced::Length::{Fill, FillPortion};
use iced::{Element, Task, Theme};

fn main() -> iced::Result {
    iced::application("Multi-Host", MultiHost::update, MultiHost::view)
        .theme(MultiHost::theme)
        .run_with(|| (MultiHost::new(), iced::Task::none()))
}

#[derive(Debug)]
struct MultiHost {
    hosted_processes: Vec<HostedProcess>,
}

#[derive(Debug)]
struct HostedProcess {
    name: String,
}

#[derive(Debug, Clone, Copy)]
enum Message {
}

impl MultiHost {
    fn new() -> Self {
        Self { 
            hosted_processes: vec![
                HostedProcess {
                    name: "Process 1".to_owned(),
                }
            ]
        }
    }

    // fn subscribe() -> Task<Message> {
    // }

    fn update(&mut self, _message: Message) -> Task<Message> {
        //match message { }
        Task::none()
    }

    fn view(&self) -> Element<Message> {

        let top_pane_text = text("top pane");
        let top_pane = container(top_pane_text)
            .width(Fill)
            .style(container::rounded_box)
            .padding(10);

        let right_pane_text = text("right pane")
            .width(Fill)
            .height(Fill);
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

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
