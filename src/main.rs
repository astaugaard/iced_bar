use iced::{
    Alignment, Background, Border, Color, Element, Font,
    Length::{self, Fill},
    Size, Task as Command, Theme,
    border::Radius,
    widget::{container, mouse_area, operation::focus, text, text_input},
};
use iced_anim::{Animated, Animation, Event, Motion};
use iced_layershell::{
    application,
    reexport::{Anchor, KeyboardInteractivity},
    settings::{LayerShellSettings, Settings, StartMode},
    to_layer_message,
};

pub fn main() -> Result<(), iced_layershell::Error> {
    let binded_output_name = std::env::args().nth(1);
    let start_mode = match binded_output_name {
        Some(output) => StartMode::TargetScreen(output),
        None => StartMode::Active,
    };

    application(Bar::default, namespace, update, view)
        .style(style)
        // .subscription(subscription)
        .settings(Settings {
            layer_settings: LayerShellSettings {
                size: Some((0, 40)),
                exclusive_zone: 40,
                anchor: Anchor::Top | Anchor::Left | Anchor::Right,
                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                start_mode,
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}

#[derive(Default)]
enum BarState {
    #[default]
    Base,
    Launch {
        command: String,
    },
}

impl BarState {
    fn get_desired_size(&self) -> Size<f32> {
        match self {
            BarState::Base => Size {
                width: 400.0,
                height: 40.0,
            },
            BarState::Launch { .. } => Size {
                width: 600.0,
                height: 500.0,
            },
        }
    }

    fn size_update(&self) -> Command<Message> {
        let size = self.get_desired_size();

        Command::done(Message::SizeUpdate(Event::Target(size)))
    }

    fn render(&self) -> Element<'_, Message> {
        match self {
            BarState::Base => text("small")
                .size(25)
                .font(Font::with_name("DejaVu Sans"))
                .into(),
            BarState::Launch { command } => container(
                text_input("Search", command)
                    .width(380)
                    .on_input(Message::CommandChanged)
                    .id("Command Input"),
            )
            .height(Length::Fixed(500.0))
            .into(),
        }
    }
}

struct Bar {
    state: BarState,

    opacity: Animated<f32>,

    previous_state: Option<(BarState, Animated<f32>)>,

    bar_size: Animated<Size<f32>>,
    command: String,
}

impl Default for Bar {
    fn default() -> Self {
        let state = BarState::default();

        let motion = Motion::SMOOTH.quick();

        Self {
            bar_size: Animated::new(state.get_desired_size(), motion),
            previous_state: None,
            state,
            command: Default::default(),
            opacity: Animated::new(1.0, motion),
        }
    }
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    HoverStart,
    HoverEnd,
    SizeUpdate(Event<Size<f32>>),
    CommandChanged(String),
}

fn size_to_window_size(size: Size<f32>) -> (u32, u32) {
    (size.width.ceil() as u32, size.height.ceil() as u32)
}

fn namespace() -> String {
    String::from("Counter - Iced")
}

fn update(bar: &mut Bar, message: Message) -> Command<Message> {
    match message {
        Message::HoverStart => {
            bar.state = BarState::Launch {
                command: String::new(),
            };
            focus("Command Input").chain(bar.state.size_update())
        }
        Message::CommandChanged(command) => {
            bar.command = command;
            Command::none()
        }
        Message::HoverEnd => {
            bar.state = BarState::Base;
            bar.command.clear();
            bar.state.size_update()
        }
        Message::SizeUpdate(event) => {
            bar.bar_size.update(event);

            match event {
                Event::Target(size) => {
                    let new = size.max(*bar.bar_size.value());

                    Command::done(Message::SizeChange(size_to_window_size(new)))
                }
                _ => {
                    if bar.bar_size.is_animating() {
                        Command::none()
                    } else {
                        Command::done(Message::SizeChange(size_to_window_size(
                            *bar.bar_size.value(),
                        )))
                    }
                }
            }
        }
        _ => Command::none(),
    }
}

fn view(bar: &Bar) -> Element<'_, Message> {
    let content = bar.state.render();

    let size = bar.bar_size.value();

    container(
        mouse_area(
            Animation::new(
                &bar.bar_size,
                container(content)
                    .style(|t: &Theme| container::Style {
                        background: Some(Background::Color(t.palette().background)),
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: Radius::new(0).bottom(25),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fixed(size.width))
                    .height(Length::Fixed(size.height))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_update(Message::SizeUpdate),
        )
        .on_enter(Message::HoverStart)
        .on_exit(Message::HoverEnd),
    )
    .align_x(Alignment::Center)
    .align_y(Alignment::Start)
    .width(Fill)
    .height(Fill)
    .into()
}

fn style(_counter: &Bar, theme: &iced::Theme) -> iced::theme::Style {
    use iced::theme::Style;
    Style {
        background_color: Color::TRANSPARENT,
        text_color: theme.palette().text,
    }
}
