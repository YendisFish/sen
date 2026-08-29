use std::sync::{Arc, Mutex, OnceLock, mpsc::Sender};

use taffy::{AlignSelf, Size, TaffyTree, style_helpers::length};
use verdant::{render_surface::RenderSurface, text::{Font, RichText, Span, TextStyle}, types::{ByteSource, Color}, vec::Vec2, window::WindowDraw};

use crate::{styling::{DisplayType, SenStyle}, views::{Component, Events, Id, SenId, Stylable, View}};

pub struct Text {
    id: Mutex<Option<SenId>>,
    text: String,
    font: Font,
    is_sized: OnceLock<Vec2>,
    style: Mutex<SenStyle>,
    size: Mutex<f32>,
}

impl Text {
    pub fn new(text: String, font: Font) -> Arc<Self> {
        Arc::new(Self {
            id: Mutex::new(None),
            text: text,
            font: font,
            is_sized: OnceLock::new(),
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
    fn click(&self) -> bool { false }
    fn drag_over(&self, view: Option<View>) {}
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
        match self.is_sized.get() {
            Some(s) => {
                let mut t = taffy::Style::DEFAULT;
                t.size = Size {
                    width: length(s.x),
                    height: length(s.y),
                };

                SenStyle {
                    taffy: t,
                    ..Default::default()
                }
            },
            None => Default::default()
        }
    }

    fn render(&self, window: &mut WindowDraw, tree: &mut TaffyTree, offset: Vec2, chan: Arc<Sender<View>>) {
        let Some(id) = self.get_id() else { return; };
        let layout = match tree.layout(id.taffy) {
            Ok(l) => l,
            Err(_) => return,
        };

        let x = offset.x + layout.location.x;
        let y = offset.y + layout.location.y;

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

        match self.is_sized.get() {
            Some(_) => {},
            None => {
                let (w, h) = window.rich_text_size(&[span.clone()]).into();
                _ = self.is_sized.set(Vec2::new(w, *size * 1.3));

                _ = chan.send(self.clone().as_view());
            },
        };

        window.rich_text(x, y, &[span]);
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
}
