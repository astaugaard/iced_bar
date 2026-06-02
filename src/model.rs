use std::fmt::Debug;

use battery::{Battery, Manager};
use chrono::Local;
use iced::{Element, Size, Task as Command};
use iced_anim::{Animated, Event, Motion};
use iced_layershell::to_layer_message;
use libpulse_binding::volume::Volume;
use smol::channel::Sender;

use crate::{
    audio_listener::AudioCommands,
    base::{get_desired_size_base, render_base},
    color::Colors,
    launch::{Launch, LaunchMessage},
};

#[derive(Default)]
pub enum BarState {
    #[default]
    Base,
    Launch(Launch),
}

impl BarState {
    pub fn get_desired_size(&self) -> Size<f32> {
        match self {
            BarState::Base => get_desired_size_base(),
            BarState::Launch(launch) => launch.get_desired_size(),
        }
        .expand(Size {
            width: 8.0,
            height: 8.0,
        })
    }

    pub fn size_update(&self) -> Command<Message> {
        let size = self.get_desired_size();

        Command::done(Message::SizeUpdate(Event::Target(size)))
    }

    pub fn render<'a>(&'a self, bar: &'a Bar) -> Element<'a, Message> {
        match self {
            BarState::Base => render_base(bar),
            BarState::Launch(launch) => launch.render(),
        }
    }
}

pub struct Bar {
    pub state: BarState,
    pub bar_size: Animated<Size<f32>>,
    pub command: String,
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
            command: Default::default(),
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

    NewVolume(Volume),

    CommandsChannel(Sender<AudioCommands>),

    Tick(chrono::DateTime<chrono::Local>),
}
