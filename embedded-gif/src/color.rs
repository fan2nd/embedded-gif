use core::marker::PhantomData;

use embedded_graphics::pixelcolor::{
    raw::{RawData, RawU1, RawU2, RawU4, RawU8},
    PixelColor,
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
