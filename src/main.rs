use std::thread;

use iced::{
    Alignment, Background, Border, Color, Element, Font,
    Length::{self, Fill},
    Size, Subscription, Task as Command, Theme,
    border::Radius,
    keyboard, stream,
    time::{self, seconds},
    widget::{container, mouse_area, operation::focus},
};
use iced_anim::{Animation, Event};
use iced_layershell::{
    application,
    reexport::{Anchor, KeyboardInteractivity, core::Widget},
    settings::{LayerShellSettings, Settings, StartMode},
};

mod audio_listener;
mod base;
mod color;
mod launch;
mod model;
mod terminal;

use iced_term::TerminalView;
use itertools::chain;
use model::Bar;
use smol::{channel::bounded, future};

use crate::{
    color::Colors,
    launch::Launch,
    model::{BarState, Message},
};

pub fn main() -> Result<(), iced_layershell::Error> {
    let binded_output_name = std::env::args().nth(1);
    let start_mode = match binded_output_name {
        Some(output) => StartMode::TargetScreen(output),
        None => StartMode::Active,
    };

    application(Bar::default, namespace, update, view)
        .style(|a, b| style(a, b, Colors::default()))
        .subscription(subscription)
        .settings(Settings {
            layer_settings: LayerShellSettings {
                size: Some((0, 48)),
                exclusive_zone: 48,
                anchor: Anchor::Top | Anchor::Left | Anchor::Right,
                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                start_mode,
                ..Default::default()
            },
            antialiasing: true,
            default_text_size: iced::Pixels(25.0),
            default_font: Font::with_name("DejaVu Sans"),
            ..Default::default()
        })
        .run()
}

fn subscription(bar: &Bar) -> Subscription<Message> {
    let sub = Subscription::batch(chain!(
        [
            time::every(seconds(2)).map(|_| Message::Tick(chrono::offset::Local::now())),
            keyboard::listen().map(|key_event| Message::KeyEvent(key_event)),
            Subscription::run(|| {
                stream::channel(8, async |mut output| {
                    thread::spawn(move || {
                        println!("making thingy");
                        let local = smol::LocalExecutor::new();

                        future::block_on({
                            local.run(async {
                                let (send_commands, commands) = bounded(8);

                                output
                                    .try_send(Message::CommandsChannel(send_commands))
                                    .unwrap();

                                audio_listener::init(output, commands).await;
                            })
                        });
                    });

                    smol::Timer::never().await;
                })
            }),
        ],
        bar.state.subscriptions()
    ));

    dbg!("subs");

    sub
}

fn size_to_window_size(size: Size<f32>) -> (u32, u32) {
    (size.width.ceil() as u32, size.height.ceil() as u32)
}

fn namespace() -> String {
    String::from("Bar - Iced")
}

fn update(bar: &mut Bar, message: Message) -> Command<Message> {
    let res = match dbg!(message) {
        Message::HoverStart => {
            Command::done(Message::StateChange(|| BarState::Launch(Launch::new())))
        }
        Message::StateChange(state) => {
            bar.state = state();

            bar.state.size_update().chain(match &bar.state {
                BarState::Launch(launch) => focus("Command Input"),
                BarState::Terminal(terminal_state) => {
                    TerminalView::focus(terminal_state.terminal.widget_id().clone())
                }
                _ => Command::none(),
            })
        }
        Message::LaunchUpdate(command) => match &mut bar.state {
            BarState::Launch(launch) => launch.update(command),
            _ => Command::none(),
        },
        Message::TerminalEvent(command) => match &mut bar.state {
            BarState::Terminal(terminal) => terminal.update(command),
            _ => Command::none(),
        },
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
                            bar.bar_size.value().clone(),
                        )))
                    }
                }
            }
        }
        Message::Tick(now) => {
            bar.now = now;

            for bat in &mut bar.batteries {
                bar.manager.refresh(bat).unwrap()
            }

            Command::none()
        }
        Message::NewVolume(volume) => {
            println!("volume: {}", volume);

            if volume.is_muted() {
                bar.volume = None;
            } else {
                bar.volume = Some(volume.print());
            }

            Command::none()
        }
        Message::KeyEvent(key) => bar.state.key_event(key),
        Message::CommandsChannel(commands) => {
            bar.commands = Some(commands);

            Command::none()
        }

        _ => Command::none(),
    };

    dbg!();

    res
}

fn view(bar: &Bar) -> Element<'_, Message> {
    let content = bar.state.render(bar);

    let size = bar.bar_size.value();

    container(
        mouse_area(
            Animation::new(
                &bar.bar_size,
                container(content)
                    .style(|_: &Theme| container::Style {
                        background: Some(Background::Color(bar.colors.background)),
                        border: Border {
                            color: bar.colors.accent,
                            width: 2.0,
                            radius: Radius::new(25),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fixed(size.width))
                    .height(Length::Fixed(size.height))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Start),
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

fn style(_counter: &Bar, _theme: &iced::Theme, colors: Colors) -> iced::theme::Style {
    use iced::theme::Style;
    Style {
        background_color: Color::TRANSPARENT,
        text_color: colors.foreground,
    }
}
