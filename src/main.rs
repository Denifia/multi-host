use std::process::{Command, Stdio};

use iced::widget;

struct Counter {
    // This will be our state of the counter app
    // a.k.a the current count value
    count: i32,
    text: String
}

#[derive(Debug, Clone, Copy)]
enum Message {
    // Emitted when the increment ("+") button is pressed
    IncrementCount,
    // Emitted when decrement ("-") button is pressed
    DecrementCount,
}

impl Counter {
    fn new() -> Self {
        // initialize the counter struct
        // with count value as 0.
        Self { count: 0, text: String::from("blank") }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        // handle emitted messages
        match message {
            Message::IncrementCount => self.count += 1,
            Message::DecrementCount => self.count -= 1,
        }
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        // create the View Logic (UI)
        let row = widget::row![
            widget::button("-").on_press(Message::DecrementCount),
            widget::text(self.count),
            widget::button("+").on_press(Message::IncrementCount),
            widget::text(self.text.clone()).height(200).width(200)
        ];
        widget::container(row)
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

fn main() -> Result<(), iced::Error> {
    let child = Command::new("cmd")
        //.arg("-Wait C:/Users/mrlwa/source/repos/multi-host/demo.log")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute child");

    let output = child
        .wait_with_output()
        .expect("failed to wait on child");

    // run the app from main function
    iced::application("Counter Example", Counter::update, Counter::view).run_with(|| (Counter::new(), iced::Task::none()))
}