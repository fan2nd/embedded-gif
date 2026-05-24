use embedded_graphics::pixelcolor::{BinaryColor, PixelColor, Rgb565, Rgb888};

pub trait PaletteColor: PixelColor {
    fn from_rgb888(red: u8, green: u8, blue: u8) -> Self;
}

impl PaletteColor for Rgb888 {
    fn from_rgb888(red: u8, green: u8, blue: u8) -> Self {
        Self::new(red, green, blue)
    }
}

impl PaletteColor for Rgb565 {
    fn from_rgb888(red: u8, green: u8, blue: u8) -> Self {
        Self::new(red >> 3, green >> 2, blue >> 3)
    }
}

impl PaletteColor for BinaryColor {
    fn from_rgb888(red: u8, green: u8, blue: u8) -> Self {
        if (u16::from(red) * 299 + u16::from(green) * 587 + u16::from(blue) * 114) / 1000 >= 128 {
            Self::On
        } else {
            Self::Off
        }
    }
}
