use iced::{Element, Font, Size, Task as Command, widget::container};
use iced_term::{ColorPalette, Terminal, TerminalView};

use crate::{
    color::Colors,
    model::{Bar, BarState, Message},
};

pub struct TerminalState {
    pub terminal: Terminal,
    pub size: Size<f32>,
}

#[derive(Debug, Clone)]
pub enum TerminalMessage {
    TerminalEvent(iced_term::Event),
}

impl TerminalState {
    pub fn render<'a>(&'a self, _bar: &'a Bar) -> Element<'a, Message> {
        container(
            TerminalView::show(&self.terminal)
                .map(|a| Message::TerminalEvent(TerminalMessage::TerminalEvent(a))),
        )
        .padding(10)
        .into()
    }

    pub fn get_desired_size(&self) -> Size<f32> {
        self.size
    }

    pub fn update(&mut self, message: TerminalMessage) -> Command<Message> {
        match message {
            TerminalMessage::TerminalEvent(iced_term::Event::BackendCall(_, cmd)) => {
                match self
                    .terminal
                    .handle(iced_term::Command::ProxyToBackend(cmd))
                {
                    iced_term::actions::Action::Shutdown => {
                        Command::done(Message::StateChange(|_| BarState::Base))
                    }
                    _ => Command::none(),
                }
            }
        }
    }

    pub fn new(command: &str, size: Size<f32>, colors: &Colors) -> Self {
        let term_settings = iced_term::settings::Settings {
            font: iced_term::settings::FontSettings {
                size: 14.0,
                font_type: Font::with_name("DejaVu Sans Mono"),
                ..Default::default()
            },
            theme: iced_term::settings::ThemeSettings::new(Box::new(ColorPalette {
                foreground: colors.foreground.to_string(),
                background: colors.background.to_string(),
                black: colors.background.to_string(),
                red: colors.color4.to_string(),
                green: colors.color3.to_string(),
                yellow: colors.color2.to_string(),
                blue: colors.color5.to_string(),
                magenta: colors.accent.to_string(),
                cyan: colors.color1.to_string(),
                white: colors.foreground.to_string(),
                bright_black: colors.background.to_string(),
                bright_red: colors.color4.to_string(),
                bright_green: colors.color3.to_string(),
                bright_yellow: colors.color2.to_string(),
                bright_blue: colors.color5.to_string(),
                bright_magenta: colors.accent.to_string(),
                bright_cyan: colors.color1.to_string(),
                bright_white: colors.foreground.to_string(),
                dim_black: colors.background.to_string(),
                dim_red: colors.color4.to_string(),
                dim_green: colors.color3.to_string(),
                dim_yellow: colors.color2.to_string(),
                dim_blue: colors.color5.to_string(),
                dim_magenta: colors.accent.to_string(),
                dim_cyan: colors.color1.to_string(),
                dim_white: colors.foreground.to_string(),
                bright_foreground: None,
                dim_foreground: colors.foreground.to_string(),
            })),
            backend: iced_term::settings::BackendSettings {
                program: command.to_string(),
                ..Default::default()
            },
        };

        let terminal = iced_term::Terminal::new(0, term_settings.clone()).unwrap();

        Self { terminal, size }
    }
}
