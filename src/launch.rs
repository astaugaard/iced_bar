
use std::fmt::Debug;

use iced::{
    Element, Length::{self},
    Size, Task as Command, widget::{container, text_input},
};

use crate::model::Message;

pub struct Launch {
    pub command: String,
}

#[derive(Debug, Clone)]
pub enum LaunchMessage {
    CommandChanged(String),
}

impl Launch {
    pub fn render(&self) -> Element<'_, Message> {
        container(
            text_input("Search", &self.command)
                .width(380)
                .on_input(|a| Message::LaunchUpdate(LaunchMessage::CommandChanged(a)))
                .id("Command Input"),
        )
        .height(Length::Fixed(500.0))
        .into()
    }

    pub fn get_desired_size(&self) -> Size<f32> {
        Size {
            width: 600.0,
            height: 500.0,
        }
    }

    pub fn update(&mut self, message: LaunchMessage) -> Command<Message> {
        match message {
            LaunchMessage::CommandChanged(command) => self.command = command,
        }

        Command::none()
    }
}

