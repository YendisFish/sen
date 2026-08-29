use std::sync::{Arc, Mutex, OnceLock, mpsc::Sender};

use taffy::{AlignSelf, Size, TaffyTree, style_helpers::length};
use verdant::{render_surface::RenderSurface, text::{Font, RichText, Span, TextStyle}, transform::Transform2d, types::{ByteSource, Color}, vec::Vec2, window::WindowDraw};

use crate::{styling::{DisplayType, Margin, Padding, SenStyle}, views::{Component, Events, Id, RenderCtx, SenId, Stylable, View}};

pub struct Text {
    id: Mutex<Option<SenId>>,
    text: String,
    font: Font,
    style: Mutex<SenStyle>,
    size: Mutex<f32>,
}

impl Text {
    pub fn new(text: String, font: Font) -> Arc<Self> {
        Arc::new(Self {
            id: Mutex::new(None),
            text: text,
            font: font,
            style: Mutex::new(Default::default()),
            size: Mutex::new(25.),
        })
    }

    pub fn text_size(self: Arc<Self>, size: f32) -> Arc<Self> {
        match self.size.lock() {
            Ok(mut val) => {
                *val = size;
            },
            Err(e) => {
                let mut val = e.into_inner();
                *val = size;
            },
        };

        self
    }
}

impl Events for Arc<Text> {
    fn click(&self, set_focus: &mut bool) -> bool { false }
    fn drag_over(&self, view: Option<View>) {}

    fn key_down(&self, key: verdant::prelude::Key) {}
}

impl Id for Arc<Text> {
    fn set_id(&self, id: SenId) {
        let mut id_guard = match self.id.lock() {
            Ok(guard) => guard,
            Err(e) => e.into_inner(),
        };

        *id_guard = Some(id);
    }

    fn get_id(&self) -> Option<SenId> {
        if let Some(id) = self.id.lock().ok() {
           *id
        } else { None }
    }
}

impl Component for Arc<Text> {
    fn as_view(self) -> View {
        Arc::new(self)
    }

    fn get_inner(&self) -> Option<Arc<Vec<View>>> {
        None
    }

    fn get_style(&self) -> SenStyle {
        match self.style.lock() {
            Ok(val) => val.clone(),
            Err(e) => {
                let val = e.into_inner();
                val.clone()
            }
        }
    }

    fn render(&self, window: &mut WindowDraw, tree: &mut TaffyTree, offset: Vec2, chan: Arc<Sender<View>>) {
        let Some(id) = self.get_id() else { return; };
        let layout = match tree.layout(id.taffy) {
            Ok(l) => l,
            Err(_) => return,
        };

        let x = offset.x + layout.location.x + layout.padding.left;
        let y = offset.y + layout.location.y + layout.padding.top;

        let color = match self.style.lock() {
            Ok(val) => match val.verdant {
                Some(v) => v.fill_color,
                None => Color::BLACK,
            }
            Err(e) => match e.into_inner().verdant {
                Some(v) => v.fill_color,
                None => Color::BLACK,
            }
        };

        let Some(size) = self.size.lock().ok() else { return; };
        let span = Span::new(self.text.clone(), self.font.clone(), TextStyle {
            size: *size,
            color: color,
            ..Default::default()
        });

        let Some(mut style) = self.style.lock().ok() else { return; };
        style.offset = Some(offset);

        window.rich_text(x, y, &[span]);
    }

    fn get_ctx(&self) -> Option<RenderCtx> {
        let color = match self.style.lock() {
            Ok(val) => match val.verdant {
                Some(v) => v.fill_color,
                None => Color::BLACK,
            }
            Err(e) => match e.into_inner().verdant {
                Some(v) => v.fill_color,
                None => Color::BLACK,
            }
        };

        let Some(size) = self.size.lock().ok() else { return None; };
        Some(RenderCtx { text: self.text.clone(), text_size: *size, font: self.font.clone(), default_color: color })
    }
}

impl Stylable for Arc<Text> {
    fn size(mut self, size: Vec2) -> Self {
        return self;
    }

    fn color(self, color: verdant::prelude::Color) -> Self {
        match self.style.lock() {
            Ok(mut val) => {
                let vd = val.verdant.get_or_insert_with(|| Default::default());
                vd.fill(color);

                val.verdant = Some(*vd);
            },
            Err(e) => {
                let mut val = e.into_inner();
                let vd = val.verdant.get_or_insert_with(|| Default::default());
                vd.fill(color);

                val.verdant = Some(*vd);
            },
        };

        self
    }

    fn display(self, tp: DisplayType) -> Self {
        self
    }

    fn align_self(self, slf: AlignSelf) -> Self {
        match self.style.lock().ok() {
            Some(mut s) => {
                s.taffy.align_self = Some(slf);
            },
            None => {},
        }

        self
    }

    fn padding(self, pad: Padding) -> Self {
        match self.style.lock().ok() {
            Some(mut style) => {
                match pad {
                    Padding::All { t, r, b, l } => {
                        style.taffy.padding = taffy::Rect {
                            left: length(l),
                            right: length(r),
                            top: length(t),
                            bottom: length(b)
                        };
                    },
                    Padding::Top { t } => {
                        style.taffy.padding.top = length(t);
                    },
                    Padding::Bottom { b } => {
                        style.taffy.padding.bottom = length(b);
                    },
                    Padding::Left { l } => {
                        style.taffy.padding.left = length(l);
                    },
                    Padding::Right { r } => {
                        style.taffy.padding.right = length(r);
                    }
                }
            }
            None => {}
        }

        self
    }

    fn margin(self, mar: Margin) -> Self {
        match self.style.lock().ok() {
            Some(mut style) => {
                match mar {
                    Margin::All { t, r, b, l } => {
                        style.taffy.margin = taffy::Rect {
                            left: length(l),
                            right: length(r),
                            top: length(t),
                            bottom: length(b)
                        };
                    },
                    Margin::Top { t } => {
                        style.taffy.margin.top = length(t);
                    },
                    Margin::Bottom { b } => {
                        style.taffy.margin.bottom = length(b);
                    },
                    Margin::Left { l } => {
                        style.taffy.margin.left = length(l);
                    },
                    Margin::Right { r } => {
                        style.taffy.margin.right = length(r);
                    }
                }
            }
            None => {}
        }

        self
    }

    fn outline(self, color: Color, size: f32) -> Self {
        match self.style.lock().ok() {
            Some(mut style) => {
                match &mut style.verdant {
                    Some(v) => {
                        v.outline_color = color;
                        v.outline_width = size;
                    },
                    None => {},
                }
            },
            None => {},
        }

        self
    }

    fn rounding(self, rounding: f32) -> Self {
        match self.style.lock().ok() {
            Some(mut style) => {
                match &mut style.verdant {
                    Some(v) => {
                        v.corner_radius = rounding;
                    },
                    None => {},
                }
            },
            None => {},
        }

        self
    }
}
