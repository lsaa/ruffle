use crate::{BlurFilter, BlurFilterFlags, Fixed8, Fixed16, GradientRecord, Rectangle, Twips};
use bitflags::bitflags;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GradientFilter {
    pub colors: Vec<GradientRecord>,
    pub blur_x: Fixed16,
    pub blur_y: Fixed16,
    pub angle: Fixed16,
    pub distance: Fixed16,
    pub strength: Fixed8,
    pub flags: GradientFilterFlags,
}

impl GradientFilter {
    #[inline]
    pub fn is_inner(&self) -> bool {
        self.flags.contains(GradientFilterFlags::INNER_SHADOW)
    }

    #[inline]
    pub fn is_knockout(&self) -> bool {
        self.flags.contains(GradientFilterFlags::KNOCKOUT)
    }

    #[inline]
    pub fn is_on_top(&self) -> bool {
        self.flags.contains(GradientFilterFlags::ON_TOP)
    }

    #[inline]
    pub fn num_passes(&self) -> u8 {
        (self.flags & GradientFilterFlags::PASSES).bits()
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        self.blur_x = BlurFilter::scale_blur(self.blur_x, x);
        self.blur_y = BlurFilter::scale_blur(self.blur_y, y);
        self.distance *= Fixed16::from_f32(y);
    }

    pub fn inner_blur_filter(&self) -> BlurFilter {
        BlurFilter {
            blur_x: self.blur_x,
            blur_y: self.blur_y,
            flags: BlurFilterFlags::from_passes(self.num_passes()),
        }
    }

    pub fn calculate_dest_rect(&self, source_rect: Rectangle<Twips>) -> Rectangle<Twips> {
        let blur_bounds = self
            .inner_blur_filter()
            .calculate_dest_rect(Rectangle::ZERO);
        let distance = self.distance.to_f64();
        let angle = self.angle.to_f64();
        // Flash pads by the blur radius and offset, plus one pixel, rounded down.
        let x = Twips::from_pixels(
            (blur_bounds.x_max.to_pixels() / 2.0 + (angle.cos() * distance).abs() + 1.0).floor(),
        );
        let y = Twips::from_pixels(
            (blur_bounds.y_max.to_pixels() / 2.0 + (angle.sin() * distance).abs() + 1.0).floor(),
        );
        Rectangle {
            x_min: source_rect.x_min - x,
            x_max: source_rect.x_max + x,
            y_min: source_rect.y_min - y,
            y_max: source_rect.y_max + y,
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct GradientFilterFlags: u8 {
        const INNER_SHADOW     = 1 << 7;
        const KNOCKOUT         = 1 << 6;
        const COMPOSITE_SOURCE = 1 << 5;
        const ON_TOP           = 1 << 4;
        const PASSES           = 0b1111;
    }
}

impl GradientFilterFlags {
    #[inline]
    pub fn from_passes(num_passes: u8) -> Self {
        let flags = Self::from_bits_retain(num_passes);
        debug_assert_eq!(flags & Self::PASSES, flags);
        flags
    }
}
