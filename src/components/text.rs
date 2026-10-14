use std::{rc::Rc, sync::{Arc, Mutex, mpsc::Sender}};

use flume::{FlumeElem, FlumeNode, PassCtx, layout};
use verdant::{render_surface::RenderSurface, text::{Font, Span, TextStyle}, types::Color, window::WindowDraw};

use crate::{SenNotif, style::SenStyle, view::{Component, Stateful, View}};

pub struct TextBox {
    id: Mutex<Option<usize>>,
    pub text: String,
    pub font: Font,
    pub style: SenStyle,
}

impl TextBox {
    pub fn new(text: impl Into<String>, font: Font) -> Rc<Self> {
        Rc::new(Self {
            id: Mutex::new(None),
            text: text.into(),
            style: SenStyle::new(),
            font: font,
        })
    }
}

impl View for TextBox {
    fn render(&self, surface: &mut WindowDraw, ctx: &mut PassCtx, sender: Arc<Sender<SenNotif>>) {
        let spans = &[
            Span {
                text: self.text.clone(),
                font: self.font.clone(),
                style: TextStyle {
                    size: 32.,
                    color: Color::WHITE,
                    ..Default::default()
                }
            },
        ];

        let (w, h) = surface.rich_text_size(spans).into();
        self.style.flume.style_with(move |_, stle| {
            stle.size.set_width(w);
            stle.size.set_height(h);
        });

        let l = layout(ctx, self, true);

        let container = ctx.inside_of.clone();
        let (c_w, c_h) = container.style_with(|_, stle| (stle.size.width(), stle.size.height()));
        let (r_w, r_h) = self.style.flume.style_with(move |_, stle| (stle.size.width(), stle.size.height()));
        if !(r_w <= c_w && r_h <= c_h) {
            eprintln!("Text \"{}\" overflowed container, displaying with warning.", self.text);
            eprintln!("Text size: {}, {}", r_w, r_h);
            eprintln!("Container size: {}, {}", c_w, c_h);
        }

        surface.rich_text(l.0, l.1, spans);
    }

    fn get_id(self: Rc<Self>) -> Option<usize> {
        match self.id.lock() {
            Ok(val) => *val,
            Err(e) => *e.into_inner(),
        }
    }

    fn set_id(self: Rc<Self>, id: usize) {
        match self.id.lock() {
            Ok(mut val) => *val = Some(id),
            Err(e) => {
                let mut val = e.into_inner();
                *val = Some(id);
            }
        }
    }

    fn is_stateful(self: Rc<Self>) -> Option<Rc<dyn Stateful>> {
        None
    }

    fn get_children(&self) -> Vec<Component> {
        vec![]
    }

    fn click(self: Rc<Self>) { }

    fn get_style(&self) -> SenStyle {
        self.style.clone()
    }

    fn color(self: Rc<Self>, color: Color) -> Component { self.clone() }
    fn size(self: Rc<Self>, size: flume::Size) -> Component { self.clone() }
}

impl FlumeNode for TextBox {
    fn get_children_nodes(&self, n: usize) -> Option<Rc<dyn FlumeNode>> { None }

    fn ctx(&self, existing: Option<&PassCtx>) -> PassCtx {
        self.style.flume.ctx(existing)
    }

    fn get_flume(&self) -> FlumeElem {
        self.style.flume.clone()
    }
}
