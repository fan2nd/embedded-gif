use core::marker::PhantomData;

use embedded_graphics::pixelcolor::{
    raw::{RawData, RawU1, RawU2, RawU4, RawU8},
    BinaryColor, PixelColor, Rgb565, Rgb888,
};

macro_rules! palette_index {
    ($name:ident, $raw:ty, $max:expr) => {
        #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $name<C> {
            index: u8,
            color: PhantomData<C>,
        }

        impl<C> $name<C> {
            pub const MAX: u8 = $max;

            pub const fn new(index: u8) -> Self {
                Self {
                    index: index & Self::MAX,
                    color: PhantomData,
                }
            }

            pub const fn index(self) -> u8 {
                self.index
            }
        }

        impl<C> PixelColor for $name<C>
        where
            C: PixelColor,
        {
            type Raw = $raw;
        }

        impl<C> From<$raw> for $name<C> {
            fn from(raw: $raw) -> Self {
                Self::new(raw.into_inner())
            }
        }
    };
}

palette_index!(PaletteIndex1, RawU1, 0x01);
palette_index!(PaletteIndex2, RawU2, 0x03);
palette_index!(PaletteIndex4, RawU4, 0x0f);
palette_index!(PaletteIndex8, RawU8, 0xff);

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
