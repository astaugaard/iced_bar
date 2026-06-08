use iced::Color;

pub struct Colors {
    pub background: Color,
    pub background2: Color,
    pub foreground: Color,
    pub color1: Color,
    pub color2: Color,
    pub accent: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            background: Color::from_rgb8(0x24, 0x27, 0x3a),
            background2: Color::from_rgb8(0x36, 0x3a, 0x4f),
            foreground: Color::from_rgb8(0xca, 0xd3, 0xf5),
            accent: Color::from_rgb8(0xc6, 0xa0, 0xf6),
            color1: Color::from_rgb8(0x8b, 0xd5, 0xca),
            color2: Color::from_rgb8(0xee, 0xd4, 0x9f),
        }
    }
}
