//#![windows_subsystem = "windows"]
// Uncomment the above before release. Prevents stupid console window.

use iced::{Element, Subscription, Task, Theme, futures::channel::mpsc::Sender};
use screens::home::HomeScreen;
use screens::settings::SettingsScreen;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::{env, io};
use thiserror::Error;
use yaml_rust2::YamlLoader;

mod hosted_process;
mod screens;

fn main() -> iced::Result {
    let args: Vec<_> = env::args().collect();
    let mut f = File::open(&args[1]).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    let docs = YamlLoader::load_from_str(s.as_str()).unwrap();
    let doc = &docs[0];

    let process_list = doc["process"].clone();
    let mut processes: Vec<ProcessDefinition> = vec![];
    for process_input in process_list.into_iter() {
        processes.push(ProcessDefinition {
            name: process_input["name"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            command: process_input["exe"]
                .as_str()
                .expect("process should have exe")
                .to_string(),
            cwd: process_input["cwd"]
                .as_str()
                .expect("process should have cwd")
                .to_string(),
            args: process_input["args"]
                .clone()
                .into_iter()
                .map(|i| i.as_str().expect("process should have args").to_string())
                .collect(),
            auto_start: process_input["auto_start"].as_bool().unwrap_or(false),
        });
    }

    let config = Configuration {
        processes: Rc::new(processes),
    };

    iced::application("Multi-Host", MultiHost::update, MultiHost::view)
        .theme(MultiHost::theme)
        .subscription(MultiHost::subscription)
        .run_with(|| (MultiHost::new(config), iced::Task::none()))
}

#[derive(Debug)]
struct Configuration {
    processes: Rc<Vec<ProcessDefinition>>,
}

#[derive(Debug, Clone)]
struct ProcessDefinition {
    name: String,
    command: String,
    cwd: String,
    args: Vec<String>,
    auto_start: bool,
}

#[derive(Debug)]
struct MultiHost {
    current_screen: Screen,
    home_screen: HomeScreen,
    settings_screen: SettingsScreen,
    output_listener: Option<Sender<Message>>,
    configuration: Rc<Configuration>,
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
    ProcessOutput(usize, String),
    StartStopProcess(usize),
    ListeningForOutput(Sender<Message>),
    FocusProcess(usize),
    AutoStartProcesses(Sender<Message>),
    ToggleHomeSideBar,
}

impl MultiHost {
    fn new(config: Configuration) -> Self {
        let p = Rc::clone(&config.processes);
        Self {
            configuration: Rc::new(config),
            current_screen: Screen::Home,
            home_screen: HomeScreen::new(p),
            settings_screen: SettingsScreen::new(),
            output_listener: None,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangeScreen(screen) => {
                self.current_screen = screen;
                Task::none()
            }
            Message::StartStopProcess(process_id) => match &self.output_listener {
                Some(listener) => self.home_screen.start_stop(process_id, listener),
                None => panic!("oh no"),
            },
            Message::ToggleHomeSideBar => self.home_screen.toggle_side_bar(),
            Message::AutoStartProcesses(sender) => self.home_screen.auto_start(&sender.clone()),
            Message::FocusProcess(process_id) => self.home_screen.focus(process_id),
            Message::ProcessOutput(_, _) => self.home_screen.update(message),
            Message::SettingsSettingOneUpdated(_) | Message::SaveSettings => {
                self.settings_screen.update(message)
            }
            Message::ListeningForOutput(sender) => {
                println!("listening for output, about to signal auto start");
                let message = Message::AutoStartProcesses(sender.clone());
                self.output_listener = Some(sender);
                Task::done(message)
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self.current_screen {
            Screen::Home => self.home_screen.view(),
            Screen::Settings => self.settings_screen.view(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let subs = self.home_screen.subscription();
        Subscription::batch(subs)
    }

    fn theme(&self) -> Theme {
        Theme::CatppuccinLatte
    }
}

#[derive(Debug, Error)]
enum MultiHostError {
    #[error("Iced error: {0}")]
    Iced(#[from] iced::Error),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Simple error: `{0}`")]
    Simple(String),
}
