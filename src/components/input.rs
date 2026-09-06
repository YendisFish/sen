use std::{fmt::format, panic, sync::{Arc, Mutex, mpsc::Sender}};

use taffy::{AlignSelf, Size, TaffyTree, style_helpers::length};
use verdant::{event::{Key, winit::NamedKey::Dead}, text::Font, types::Color, vec::Vec2, window::WindowDraw};

use crate::{components::{bind::State, div::Div, many::Many, text::Text}, many, styling::{DisplayType, Margin, Padding, SenStyle}, views::{Component, Events, Id, SenId, Stylable, View}};

fn default_style() -> SenStyle {
    let mut tffy: taffy::Style<String> = taffy::Style::default();
    tffy.size = Size {
        width: length(200.),
        height: length(30.)
    };

    let ret = SenStyle {
        verdant: Some(Default::default()),
        offset: None,
        taffy: Arc::new(Mutex::new(tffy))
    };


    ret
}

pub struct Input {
    id: Mutex<Option<SenId>>,
    data: Arc<State<String>>,
    style: Arc<Mutex<SenStyle>>,
    font: Font,
    on_submit: Option<Box<dyn Fn(Arc<State<String>>) + Send + Sync>>,
}

impl Input {
    pub fn new(font: Font, state: Option<Arc<State<String>>>) -> Self {
        let m_font = font.clone();

        let ret = Self {
            id: Mutex::new(None),
            data: state.unwrap_or(State::new(String::new())),
            style: Arc::new(Mutex::new(default_style())),
            font: font,
            on_submit: None,
        };

        let style_arc = ret.style.clone();

        ret.data.with(move |dat| {
            let (width, height, color, outline_color, outline_size) = match style_arc.lock() {
                Ok(val) => (
                    val.taffy().size.width.value(),
                    val.taffy().size.height.value(),
                    match val.verdant {
                        Some(v) => v.fill_color,
                        None => default_style().verdant.unwrap().fill_color,
                    },
                    match val.verdant {
                        Some(v) => v.outline_color,
                        None => default_style().verdant.unwrap().outline_color,
                    },
                    match val.verdant {
                        Some(v) => v.outline_width,
                        None => default_style().verdant.unwrap().outline_width,
                    },
                ),
                Err(e) => {
                    let val = e.into_inner();
                    (
                        val.taffy().size.height.value(),
                        val.taffy().size.width.value(),
                        match val.verdant {
                            Some(v) => v.fill_color,
                            None => default_style().verdant.unwrap().fill_color,
                        },
                        match val.verdant {
                            Some(v) => v.outline_color,
                            None => default_style().verdant.unwrap().outline_color,
                        },
                        match val.verdant {
                            Some(v) => v.outline_width,
                            None => default_style().verdant.unwrap().outline_width,
                        },
                    )
                }
            };

            let dv = Div::new(
                Text::new(dat.clone(), m_font.clone()).text_size(height / 1.3)
            )
            .color(color)
            .align_self(AlignSelf::STRETCH)
            .size(Vec2::new(width, height))
            .outline(outline_color, outline_size);

            dv.as_view()
        });

        ret
    }

    pub fn on_submit(mut self, fun: impl Fn(Arc<State<String>>) + 'static + Send + Sync) -> Self {
        self.on_submit = Some(Box::new(fun));
        self
    }
}

impl Id for Input {
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

impl Events for Input {
    fn click(&self, set_focus: &mut bool) -> bool {
        *set_focus = true;
        true
    }
    fn drag_over(&self, view: Option<View>) {}

    fn key_down(&self, key: verdant::prelude::Key) {
        let mut old_data = self.data.get().clone();

        match key {
            verdant::prelude::Key::Character(c) => {
                let new_data = old_data + c.as_str();
                _ = self.data.set(new_data);
            },
            verdant::prelude::Key::Backspace => {
                old_data.pop();
                _ = self.data.set(old_data);
            }
            verdant::prelude::Key::Enter => {
                let Some(c) = &self.on_submit else { return; };
                c(self.data.clone());
            },
            _ => {},
        }
    }
}

impl Component for Input {
    fn as_view(self) -> View {
        Arc::new(self)
    }

    fn get_ctx(&self) -> Option<crate::views::RenderCtx> { None }

    fn get_inner(&self) -> Option<Arc<Vec<View>>> {
        Some(Arc::new(vec![self.data.clone().as_view()]))
    }

    fn get_style(&self) -> SenStyle {
        match self.style.lock() {
            Ok(val) => val.clone(),
            Err(e) => e.into_inner().clone(),
        }
    }

    fn render(&self, window: &mut WindowDraw, tree: &mut TaffyTree, mut offset: Vec2, chan: Arc<Sender<View>>) {
        let Some(id) = self.get_id() else { return; };
        let layout = match tree.layout(id.taffy) {
            Ok(l) => l,
            Err(_) => return,
        };

        let x = offset.x + layout.location.x;
        let y = offset.y + layout.location.y;

        offset.x = x;
        offset.y = y;

        self.data.render(window, tree, offset, chan.clone());
    }
}

impl Stylable for Input {
    fn size(self, size: Vec2) -> Self {
        match self.style.lock() {
            Ok(mut val) => {
                val.taffy().size = taffy::Size {
                    width: length(size.x),
                    height: length(size.y),
                };
            },
            Err(e) => {
                let mut val = e.into_inner();

                val.taffy().size = taffy::Size {
                    width: length(size.x),
                    height: length(size.y),
                };
            },
        };

        self.data.recompute();
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

        self.data.recompute();
        self
    }

    fn display(self, tp: DisplayType) -> Self {
        self.data.recompute();
        self
    }

    fn align_self(self, slf: AlignSelf) -> Self {
        match self.style.lock().ok() {
            Some(mut s) => {
                s.taffy().align_self = Some(slf);
            },
            None => {},
        }

        self.data.recompute();
        self
    }

    fn padding(self, pad: Padding) -> Self {
        match self.style.lock().ok() {
            Some(mut style) => {
                match pad {
                    Padding::All { t, r, b, l } => {
                        style.taffy().padding = taffy::Rect {
                            left: length(l),
                            right: length(r),
                            top: length(t),
                            bottom: length(b)
                        };
                    },
                    Padding::Top { t } => {
                        style.taffy().padding.top = length(t);
                    },
                    Padding::Bottom { b } => {
                        style.taffy().padding.bottom = length(b);
                    },
                    Padding::Left { l } => {
                        style.taffy().padding.left = length(l);
                    },
                    Padding::Right { r } => {
                        style.taffy().padding.right = length(r);
                    }
                }
            }
            None => {}
        }

        self.data.recompute();
        self
    }

    fn margin(self, mar: Margin) -> Self {
        match self.style.lock().ok() {
            Some(mut style) => {
                match mar {
                    Margin::All { t, r, b, l } => {
                        style.taffy().margin = taffy::Rect {
                            left: length(l),
                            right: length(r),
                            top: length(t),
                            bottom: length(b)
                        };
                    },
                    Margin::Top { t } => {
                        style.taffy().margin.top = length(t);
                    },
                    Margin::Bottom { b } => {
                        style.taffy().margin.bottom = length(b);
                    },
                    Margin::Left { l } => {
                        style.taffy().margin.left = length(l);
                    },
                    Margin::Right { r } => {
                        style.taffy().margin.right = length(r);
                    }
                }
            }
            None => {}
        }

        self.data.recompute();
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

        self.data.recompute();
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

        self.data.recompute();
        self
    }
}
