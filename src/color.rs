use iced::Color;

pub struct Colors {
    pub background: Color,
    pub foreground: Color,
    pub color1: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            background: Color::from_rgb8(0x24, 0x27, 0x3a),
            foreground: Color::from_rgb8(0xca, 0xd3, 0xf5),
            color1: Color::from_rgb8(0x8b, 0xd5, 0xca),
        }
    }
}
