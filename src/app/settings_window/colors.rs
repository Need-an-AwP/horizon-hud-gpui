use std::ops::Deref;

use gpui::{Rgba, Window, colors::Colors as BaseColors, rgb};

pub struct Colors {
    base: BaseColors,
    pub warning: Rgba,
    pub success: Rgba,
}

impl Colors {
    pub const WARNING: u32 = 0xffcc22;
    pub const SUCCESS: u32 = 0x22cc44;

    pub fn for_appearance(window: &Window) -> Self {
        Self::from_base(BaseColors::for_appearance(window))
    }

    pub fn light() -> Self {
        Self::from_base(BaseColors::light())
    }

    pub fn dark() -> Self {
        Self::from_base(BaseColors::dark())
    }

    fn from_base(base: BaseColors) -> Self {
        Self {
            base,
            warning: rgb(Self::WARNING),
            success: rgb(Self::SUCCESS),
        }
    }
}

impl Deref for Colors {
    type Target = BaseColors;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
