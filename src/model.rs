use std::fmt::Debug;

use battery::{Battery, Manager};
use chrono::Local;
use iced::{Element, Size, Subscription, Task as Command, keyboard};
use iced_anim::{Animated, Event, Motion};
use iced_layershell::to_layer_message;
use libpulse_binding::volume::Volume;
use tokio::sync::mpsc::Sender;

use crate::{
    audio_listener::AudioCommands,
    base::{get_desired_size_base, render_base},
    color::Colors,
    launch::{Launch, LaunchMessage},
    terminal::{TerminalMessage, TerminalState},
};

#[derive(Default)]
pub enum BarState {
    #[default]
    Base,
    Launch(Launch),
    Terminal(TerminalState),
}

impl BarState {
    pub fn get_desired_size(&self) -> Size<f32> {
        match self {
            BarState::Base => get_desired_size_base(),
            BarState::Launch(launch) => launch.get_desired_size(),
            BarState::Terminal(terminal) => terminal.get_desired_size(),
        }
    }

    pub fn size_update(&self) -> Command<Message> {
        let size = self.get_desired_size();

        Command::done(Message::SizeUpdate(Event::Target(size)))
    }

    pub fn render<'a>(&'a self, bar: &'a Bar) -> Element<'a, Message> {
        match self {
            BarState::Base => render_base(bar),
            BarState::Launch(launch) => launch.render(bar),
            BarState::Terminal(terminal_state) => terminal_state.render(bar),
        }
    }

    pub fn subscriptions(&self) -> Option<Subscription<Message>> {
        match self {
            BarState::Terminal(terminal_state) => {
                Some(
                    terminal_state
                        .terminal
                        .subscription()
                        .map(|a| Message::TerminalEvent(TerminalMessage::TerminalEvent(a))),
                )
            }
            _ => None,
        }
    }

    pub fn key_event(&mut self, key: keyboard::Event) -> Command<Message> {
        match self {
            BarState::Base => Command::none(),
            BarState::Launch(launch) => launch.key_event(key),
            BarState::Terminal(_terminal_state) => Command::none(),
        }
    }
}

pub struct Bar {
    pub state: BarState,
    pub bar_size: Animated<Size<f32>>,
    pub now: chrono::DateTime<Local>,
    pub colors: Colors,
    pub commands: Option<Sender<AudioCommands>>,
    pub volume: Option<String>,
    pub batteries: Vec<Battery>,
    pub manager: Manager,
}

impl Default for Bar {
    fn default() -> Self {
        let state = BarState::Base;

        let motion = Motion::SMOOTH.quick();

        let manager = battery::Manager::new().unwrap();

        let batteries = manager
            .batteries()
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        Self {
            bar_size: Animated::new(state.get_desired_size(), motion),
            state,
            now: chrono::offset::Local::now(),
            colors: Default::default(),
            commands: Default::default(),
            volume: None,
            batteries,
            manager,
        }
    }
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    HoverStart,
    HoverEnd,
    SizeUpdate(Event<Size<f32>>),

    LaunchUpdate(LaunchMessage),

    TerminalEvent(TerminalMessage),

    NewVolume(Volume),

    CommandsChannel(Sender<AudioCommands>),

    StateChange(fn(&Bar) -> BarState),

    KeyEvent(keyboard::Event),

    Tick(chrono::DateTime<chrono::Local>),
}
