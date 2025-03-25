use iced::widget::{column, container, row, text};
use iced::Length::{Fill, FillPortion};
use iced::{Element, Length, Task, Theme};

fn main() -> iced::Result {
    iced::application("Multi-Host", MultiHost::update, MultiHost::view)
        .theme(MultiHost::theme)
        .run_with(|| (MultiHost::new(), iced::Task::none()))
}

#[derive(Debug)]
struct MultiHost {
}

#[derive(Debug, Clone, Copy)]
enum Message {
}

impl MultiHost {
    fn new() -> Self {
        Self { }
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

        let right_pane_text = text("right pane");
        let right_pane = container(right_pane_text)
            .width(FillPortion(4))
            .height(Fill)
            .padding(10);

        let left_pane_text = text("left pane");
        let left_pane = container(left_pane_text)
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
