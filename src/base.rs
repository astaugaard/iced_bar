use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Size,
    widget::{row, space, text},
};

use crate::model::{Bar, Message};

pub fn render_base(bar: &Bar) -> Element<'_, Message> {
    let time = bar.now.format("%a %b %e %H:%M");

    row![
        text(match &bar.volume {
            Some(volume) => {
                volume.clone()
            }
            None => {
                "󰖁".to_string()
            }
        })
        .color(bar.colors.color2),
        space().width(10),
        text(time.to_string()).color(bar.colors.color1)
    ]
    .height(Fill)
    .align_y(Center)
    .into()
}

pub fn get_desired_size_base() -> Size<f32> {
    Size {
        width: 400.0,
        height: 40.0,
    }
}
