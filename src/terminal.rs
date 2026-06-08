use iced::{Element, Font, Size, Task as Command};
use iced_term::{BackendCommand, Terminal, TerminalView};

use crate::model::{Bar, BarState, Message};

pub struct TerminalState {
    pub terminal: Terminal,
    pub size: Size<f32>,
}

#[derive(Debug, Clone)]
pub enum TerminalMessage {
    TerminalEvent(iced_term::Event),
}

impl TerminalState {
    pub fn render<'a>(&'a self, bar: &'a Bar) -> Element<'a, Message> {
        TerminalView::show(&self.terminal)
            .map(|a| Message::TerminalEvent(TerminalMessage::TerminalEvent(a)))
    }

    pub fn get_desired_size(&self) -> Size<f32> {
        self.size
    }

    pub fn update(&mut self, message: TerminalMessage) -> Command<Message> {
        match message {
            TerminalMessage::TerminalEvent(iced_term::Event::BackendCall(_, cmd)) => match self
                .terminal
                .handle(iced_term::Command::ProxyToBackend(cmd))
            {
                iced_term::actions::Action::Shutdown => {
                    dbg!("shutdown");

                    Command::done(Message::StateChange(|| BarState::Base))
                }
                _ => Command::none(),
            },
        }
    }

    pub fn new(command: &str, size: Size<f32>) -> Self {
        let term_settings = iced_term::settings::Settings {
            font: iced_term::settings::FontSettings {
                size: 14.0,
                font_type: Font::with_name("DejaVu Sans"),
                ..Default::default()
            },
            theme: iced_term::settings::ThemeSettings::default(),
            backend: iced_term::settings::BackendSettings {
                program: command.to_string(),
                ..Default::default()
            },
        };

        let terminal = iced_term::Terminal::new(0, term_settings.clone()).unwrap();

        Self { terminal, size }
    }
}
