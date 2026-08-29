use std::sync::{Arc, Mutex, RwLock, mpsc::Sender};

use taffy::{AlignSelf, Size, TaffyTree, style_helpers::length};
use verdant::{shapes::{Drawable, Rect, Style}, types::Color, vec::Vec2, window::WindowDraw};

use crate::{styling::{DisplayType, Margin, Padding, SenStyle}, ui::SenWindow, views::{Component, Events, Id, SenId, Stylable, View}};

fn default_style() -> SenStyle {
    let mut r = SenStyle {
        verdant: Some(Style {
            fill_color: Color::BLACK,
            ..Default::default()
        }),
        offset: None,
        ..Default::default()
    };

    r.taffy.align_self = Some(AlignSelf::STRETCH);

    r
}

pub struct Div {
    id: Mutex<Option<SenId>>,
    inner: Arc<Vec<View>>,
    style: Mutex<SenStyle>,
    on_click: Option<Box<dyn Fn()>>,
}

impl Div {
    pub fn new(children: impl Component) -> Self {
        Self {
            id: Mutex::new(None),
            inner: Arc::new(vec![children.as_view()]),
            style: Mutex::new(default_style()),
            on_click: None,
        }
    }

    pub fn on_click(mut self, fun: impl Fn() + 'static) -> Self {
        self.on_click = Some(Box::new(fun));
        self
    }
}

impl Id for Div {
    fn set_id(&self, id: SenId) {
        let mut id_guard = match self.id.lock() {
            Ok(guard) => guard,
            Err(e) => e.into_inner(),
        };

        *id_guard = Some(id);
    }

    fn get_id(&self) -> Option<crate::views::SenId> {
        if let Some(id) = self.id.lock().ok() {
           *id
        } else { None }
    }
}

impl Component for Div {
    fn get_inner(&self) -> Option<Arc<Vec<View>>> {
        Some(self.inner.clone())
    }

    fn as_view(self) -> View {
       Arc::new(self)
    }

    fn render(&self, window: &mut WindowDraw, tree: &mut TaffyTree, mut offset: Vec2, chan: Arc<Sender<View>>) {
        let Some(id) = self.get_id() else { return; };
        let layout = match tree.layout(id.taffy) {
            Ok(l) => l,
            Err(_) => return,
        };

        let Some(mut style) = self.style.lock().ok() else { return; };
        let verdant = match style.verdant {
            Some(v) => v,
            None => default_style().verdant.unwrap(),
        };

        let x = offset.x + layout.location.x;
        let y = offset.y + layout.location.y;
        Rect::at(x, y)
            .fill(verdant.fill_color)
            .size(layout.size.width, layout.size.height)
            .corner_radius(verdant.corner_radius)
            .outline(verdant.outline_color, verdant.outline_width)
            .draw(window);

        offset.x = x;
        offset.y = y;

        style.offset = Some(offset);
        for c in self.inner.as_ref() {
            c.render(window, tree, offset, chan.clone());
        }
    }

    fn get_style(&self) -> SenStyle {
        match self.style.lock() {
            Ok(val) => val.clone(),
            Err(e) => e.into_inner().clone(),
        }
    }
}

impl Stylable for Div {
    fn size(mut self, size: Vec2) -> Self {
        match self.style.lock() {
            Ok(mut val) => {
                val.taffy.size = taffy::Size {
                    width: length(size.x),
                    height: length(size.y),
                };
            },
            Err(e) => {
                let mut val = e.into_inner();

                val.taffy.size = taffy::Size {
                    width: length(size.x),
                    height: length(size.y),
                };
            },
        };

        return self;
    }

    fn color(self, color: verdant::prelude::Color) -> Self {
        match self.style.lock() {
            Ok(mut val) => {
                let vd = val.verdant.get_or_insert_with(|| default_style().verdant.unwrap());
                vd.fill(color);

                val.verdant = Some(*vd);
            },
            Err(e) => {
                let mut val = e.into_inner();
                let vd = val.verdant.get_or_insert_with(|| default_style().verdant.unwrap());
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

impl Events for Div {
    fn click(&self) -> bool {
        let Some(c) = &self.on_click else { return false; };
        c();

        true
    }

    fn drag_over(&self, view: Option<View>) {}
}
