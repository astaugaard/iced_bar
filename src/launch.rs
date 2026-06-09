use std::{collections::BTreeMap, fmt::Debug};

use iced::{
    Color, Element, Point, Renderer, Size, Task as Command,
    border::{Border, Radius},
    keyboard::{self, Key, key::Named},
    widget::{canvas, canvas::Path, column, container, text_input},
};
use iced_anim::{AnimationBuilder, Motion};
use once_cell::sync::Lazy;

use crate::{
    model::{Bar, BarState, Message},
    terminal::TerminalState,
};

static OPTIONS: Lazy<BTreeMap<String, fn(&Bar) -> BarState>> = Lazy::new(|| {
    let mut map: BTreeMap<String, fn(&Bar) -> BarState> = BTreeMap::new();

    map.insert("Wifi".to_string(), |bar| {
        BarState::Terminal(TerminalState::new(
            "impala",
            Size {
                width: 800.0,
                height: 600.0,
            },
            &bar.colors,
        ))
    });
    map.insert("Bluetooth".to_string(), |bar| {
        BarState::Terminal(TerminalState::new(
            "bluetui",
            Size {
                width: 800.0,
                height: 600.0,
            },
            &bar.colors,
        ))
    });
    map.insert("Power".to_string(), |_| BarState::Base);

    map
});

#[derive(Clone, Debug)]
struct OptionsList<'a, A>
where
    A: Iterator<Item = &'a str> + Clone,
{
    items: A,
    selector_location: f32,
    foreground: Color,
    background: Color,
    selection: Color,
}

impl<'a, A> canvas::Program<Message> for OptionsList<'a, A>
where
    A: Iterator<Item = &'a str> + Clone,
{
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced_layershell::reexport::core::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        for (idx, name) in self.items.clone().take(7).enumerate() {
            frame.fill(
                &Path::rounded_rectangle(
                    Point {
                        x: 0.0,
                        y: 55.0 * (idx as f32),
                    },
                    Size {
                        width: 380.0,
                        height: 45.0,
                    },
                    Radius::new(25),
                ),
                self.background,
            );

            frame.fill_text(canvas::Text {
                content: name.to_string(),
                position: Point {
                    x: 10.0,
                    y: 55.0 * (idx as f32) + 22.5,
                },
                max_width: 360.0,
                color: self.foreground,
                align_x: iced::widget::text::Alignment::Left,
                align_y: iced::alignment::Vertical::Center,
                ..Default::default()
            });
        }

        // for ele in self.items.take(7).enumerate().map(|(idx, _)| {
        //     Path::rounded_rectangle(
        //         Point {
        //             x: 0.0,
        //             y: 55.0 * (idx as f32),
        //         },
        //         Size {
        //             width: 380.0,
        //             height: 45.0,
        //         },
        //         Radius::new(25),
        //     )
        // }) {
        //     frame.fill(&ele, self.background);
        // }

        // for ele in self
        //     .items
        //     .take(7)
        //     .enumerate()
        //     .map(|(idx, name)| canvas::Text {
        //         content: name.to_string(),
        //         position: Point {
        //             x: 10.0,
        //             y: 55.0 * (idx as f32) + 22.5,
        //         },
        //         max_width: 360.0,
        //         color: self.foreground,
        //         align_x: iced::widget::text::Alignment::Left,
        //         align_y: iced::alignment::Vertical::Center,
        //         ..Default::default()
        //     })
        // {
        //     frame.fill_text(ele);
        // }

        frame.fill(
            &Path::rounded_rectangle(
                Point {
                    x: 0.0,
                    y: 55.0 * self.selector_location,
                },
                Size {
                    width: 380.0,
                    height: 45.0,
                },
                Radius::new(25),
            ),
            self.selection,
        );

        vec![frame.into_geometry()]
    }
}

#[derive(Clone, Debug)]
pub struct Launch {
    pub command: String,
    pub current_list: Vec<(String, fn(&Bar) -> BarState)>,
    pub selection: usize,
}

#[derive(Debug, Clone)]
pub enum LaunchMessage {
    CommandChanged(String),
}

impl Launch {
    pub fn render<'a>(&'a self, bar: &'a Bar) -> Element<'a, Message> {
        column([
            container(
                text_input("Search", &self.command)
                    .width(380)
                    .on_input(|a| Message::LaunchUpdate(LaunchMessage::CommandChanged(a)))
                    // .icon(Icon {
                    //     font: todo!(),
                    //     code_point: todo!(),
                    //     size: todo!(),
                    //     spacing: todo!(),
                    //     side: todo!()
                    // })
                    .padding(10.0)
                    .style(|_, _| text_input::Style {
                        background: iced::Background::Color(bar.colors.background2),
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: Radius::new(25),
                        },
                        placeholder: bar.colors.foreground,
                        value: bar.colors.foreground,
                        icon: bar.colors.foreground,
                        selection: bar.colors.color1,
                    })
                    .id("Command Input"),
            )
            .into(),
            AnimationBuilder::new(self.selection as f32, |location| {
                canvas(OptionsList {
                    items: self.current_list.iter().map(|(a, _)| a.as_str()),
                    selector_location: location,
                    foreground: bar.colors.foreground,
                    background: bar.colors.background2,
                    selection: bar.colors.accent,
                })
                .height(385.0)
                .width(380.0)
                .into()
            })
            .animation(Motion::SMOOTH.quick())
            .into(),
        ])
        .spacing(10.0)
        .padding(10.0)
        .height(self.get_desired_size().height)
        .clip(true)
        .into()
    }

    pub fn get_desired_size(&self) -> Size<f32> {
        Size {
            width: 400.0,
            height: (self.current_list.len() as f32) * (55.0) + 75.0,
        }
    }

    pub fn update(&mut self, message: LaunchMessage) -> Command<Message> {
        match message {
            LaunchMessage::CommandChanged(command) => {
                self.current_list = OPTIONS
                    .iter()
                    .filter(|(name, _)| name.to_lowercase().contains(command.as_str()))
                    .map(|(a, b)| (a.clone(), b.clone()))
                    .collect();

                let max = if self.current_list.len() == 0 {
                    0
                } else {
                    self.current_list.len() - 1
                };

                self.selection = self.selection.min(max.min(6));

                self.command = command;

                Command::done(Message::SizeUpdate(iced_anim::Event::Target(
                    self.get_desired_size(),
                )))
            }
        }
    }

    pub fn new() -> Self {
        Self {
            command: String::new(),
            current_list: OPTIONS
                .iter()
                .map(|(a, b)| (a.clone(), b.clone()))
                .collect(),
            selection: 0,
        }
    }

    pub fn key_event(&mut self, key: iced::keyboard::Event) -> Command<Message> {
        match key {
            keyboard::Event::KeyPressed { key, .. } => match key {
                Key::Named(Named::ArrowDown) => {
                    let max = if self.current_list.len() == 0 {
                        1
                    } else {
                        self.current_list.len()
                    };

                    self.selection += 1;
                    self.selection %= max.min(7);

                    Command::none()
                }
                Key::Named(Named::ArrowUp) => {
                    self.selection = if self.selection == 0 {
                        if self.current_list.len() == 0 {
                            0
                        } else {
                            self.current_list.len() - 1
                        }
                    } else {
                        self.selection - 1
                    };

                    Command::none()
                }
                Key::Named(Named::Enter) => {
                    if self.current_list.len() > 0 {
                        Command::done(Message::StateChange(
                            self.current_list[self.selection].1.clone(),
                        ))
                    } else {
                        Command::none()
                    }
                }
                _ => Command::none(),
            },
            _ => Command::none(),
        }
    }
}
