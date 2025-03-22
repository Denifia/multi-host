use iced::widget;

fn main() -> Result<(), iced::Error> {
    iced::application("Multi-Host", App::update, App::view)
        .run_with(|| (App::new(), iced::Task::none()))
}

struct App {
}

#[derive(Debug, Clone, Copy)]
enum Message {
}

impl App {
    fn new() -> Self {
        Self { }
    }

    fn update(&mut self, _message: Message) -> iced::Task<Message> {
        //match message { }
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let text = widget::text(String::from("Hello, world."));
        widget::container(text)
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}
