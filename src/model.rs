use std::fmt::Debug;

use chrono::Local;
use iced::{Element, Size, Task as Command};
use iced_anim::{Animated, Event, Motion};
use iced_layershell::to_layer_message;

use crate::{
    base::{get_desired_size_base, render_base},
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
}

impl Default for Bar {
    fn default() -> Self {
        let state = BarState::Base;

        let motion = Motion::SMOOTH.quick();

        Self {
            bar_size: Animated::new(state.get_desired_size(), motion),
            state,
            command: Default::default(),
            now: chrono::offset::Local::now(),
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

    Tick(chrono::DateTime<chrono::Local>),
}
