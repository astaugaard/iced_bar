use iced::{Element, Font, Size, widget::text};

use crate::model::{Bar, Message};

pub fn render_base(bar: &Bar) -> Element<'_, Message> {
    let time = bar.now.format("%a %b %e %H:%M");

    text(time.to_string())
        .size(25)
        .font(Font::with_name("DejaVu Sans"))
        .into()
}

pub fn get_desired_size_base() -> Size<f32> {
    Size {
        width: 400.0,
        height: 40.0,
    }
}
