use taffy::{AlignSelf, FlexDirection, Style};
use verdant::vec::Vec2;


#[derive(Clone)]
pub struct SenStyle {
    pub verdant: Option<verdant::shapes::Style>,
    pub offset: Option<Vec2>,
    pub taffy: taffy::Style<String>,
}

pub enum DisplayType {
    Flex(FlexDirection),
    Grid()
}

pub enum Padding {
    All { t: f32, r: f32, b: f32, l: f32 },
    Top {t: f32 },
    Bottom { b: f32 },
    Left { l: f32 },
    Right { r: f32 },
}

pub enum Margin {
    All { t: f32, r: f32, b: f32, l: f32 },
    Top {t: f32 },
    Bottom { b: f32 },
    Left { l: f32 },
    Right { r: f32 },
}

impl Padding {
    pub fn uniform(size: f32) -> Self {
        Padding::All { t: size, r: size, b: size, l: size }
    }

    pub fn left_right(l: f32, r: f32) -> Self {
        Padding::All { t: 0., r: r, b: 0., l: l }
    }

    pub fn all(t: f32, r: f32, b: f32, l: f32) -> Self {
        Padding::All { t, r, b, l }
    }
}

impl Margin {
    pub fn uniform(size: f32) -> Self {
        Margin::All { t: size, r: size, b: size, l: size }
    }

    pub fn left_right(l: f32, r: f32) -> Self {
        Margin::All { t: 0., r: r, b: 0., l: l }
    }

    pub fn all(t: f32, r: f32, b: f32, l: f32) -> Self {
        Margin::All { t, r, b, l }
    }
}

impl Default for SenStyle {
    fn default() -> Self {
        let mut t: Style<String> = Style::DEFAULT;
        t.align_self = Some(AlignSelf::FLEX_START);

        SenStyle {
            verdant: None,
            offset: None,
            taffy: t,
        }
    }
}
