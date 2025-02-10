use embedded_graphics_core::{geometry::Size, primitives::Rectangle};
use embedded_graphics_core::{
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor},
};
use embedded_hal::digital::OutputPin;

use crate::dcs::BitsPerPixel;
use crate::dcs::InterfaceExt;
use crate::interface::{self, AsyncInterface, ContextInterface, FlushingInterface};
use crate::{dcs::WriteMemoryStart, models::Model};
use crate::{interface::InterfacePixelFormat, Display};

impl<DI, M, RST> Display<DI, M, RST>
where
    DI: AsyncInterface + ContextInterface + FlushingInterface,
    M: Model,
    M::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
    Rgb565: InterfacePixelFormat<<DI as AsyncInterface>::Word>,
{
    pub async fn fill_solid(&mut self, area: &Rectangle, color: Rgb565) -> Result<(), DI::Error> {
        let area = area.intersection(&Rectangle::new(Point::new(0, 0), Size::new(240, 320)));
        let Some(bottom_right) = area.bottom_right() else {
            // No intersection -> nothing to draw
            return Ok(());
        };

        let count = area.size.width * area.size.height;

        let sx = area.top_left.x as u16;
        let sy = area.top_left.y as u16;
        let ex = bottom_right.x as u16;
        let ey = bottom_right.y as u16;

        self.set_address_window(sx, sy, ex, ey).await?;
        self.di.write_command(WriteMemoryStart).await?;
        Rgb565::send_repeated_pixel(&mut self.di, color, count)
    }

    pub async fn set_context(&mut self, area: &Rectangle) -> Result<(), DI::Error> {
        let area = area.intersection(&Rectangle::new(Point::new(0, 0), Size::new(240, 320)));
        let Some(bottom_right) = area.bottom_right() else {
            // No intersection -> nothing to draw
            return Ok(());
        };

        let sx = area.top_left.x as u16;
        let sy = area.top_left.y as u16;
        let ex = bottom_right.x as u16;
        let ey = bottom_right.y as u16;

        self.set_address_window(sx, sy, ex, ey).await?;
        self.di.write_command(WriteMemoryStart).await?;

        Ok(())
    }
    pub async fn fill_solid_in_context(
        &mut self,
        area: &Rectangle,
        color: Rgb565,
    ) -> Result<(), DI::Error> {
        self.di
            .send_repeated_pixel_in_context(
                interface::rgb565_to_bytes(color),
                area.top_left.y as usize,
                area.top_left.x as usize,
                area.size.height as usize,
                area.size.width as usize,
            )
            .unwrap();

        self.di.write_command(WriteMemoryStart).await?;
        Ok(())
    }
}

impl BitsPerPixel {
    /// Returns the bits per pixel for a embedded-graphics [`RgbColor`].
    pub const fn from_rgb_color<C: RgbColor>() -> Self {
        let bpp = C::MAX_R.trailing_ones() + C::MAX_G.trailing_ones() + C::MAX_B.trailing_ones();

        match bpp {
            3 => Self::Three,
            8 => Self::Eight,
            12 => Self::Twelve,
            16 => Self::Sixteen,
            18 => Self::Eighteen,
            24 => Self::TwentyFour,
            _ => panic!("invalid RgbColor bits per pixel"),
        }
    }
}
