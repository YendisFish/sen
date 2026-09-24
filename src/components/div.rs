use std::{rc::Rc, sync::{Arc, Mutex, OnceLock, mpsc::Sender}};

use flume::{FlumeElem, FlumeNode, PassCtx, Size, layout};
use verdant::{render_surface::RenderSurface, shapes::{Drawable, Rect}, types::Color, vec, window::WindowDraw};

use crate::{SenNotif, style::SenStyle, view::{Component, Stateful, View}};

fn default_style() -> SenStyle {
    let stl = SenStyle::new();
    stl.verdant().fill(Color::RED);

    stl.flume.style_with(|_, fun| {
        fun.size = Size::take(10., 10.);
    });

    stl
}

pub struct Div {
    id: Mutex<Option<usize>>,
    style: SenStyle,
    children: Vec<Component>,
    event_on_click: Mutex<Option<Box<dyn Fn()>>>,
}

impl Div {
    pub fn new(c: Vec<Component>) -> Rc<Self> {
        Rc::new(Self {
            id: Mutex::new(None),
            style: default_style(),
            children: c,
            event_on_click: Mutex::new(None),
        })
    }

    pub fn on_click(self: Rc<Self>, fun: impl Fn() + 'static) -> Rc<Self> {
        let mut e_click = match self.event_on_click.lock() {
            Ok(v) => v,
            Err(e) => e.into_inner(),
        };

        *e_click = Some(Box::new(fun));

        self.clone()
    }
}

impl View for Div {
    fn render(&self, surface: &mut WindowDraw, ctx: &mut PassCtx, sender: Arc<Sender<SenNotif>>) {
        let children_current = ctx.clone();
        let l = layout(ctx, self, true);

        let total = self.style.flume.style_with(|_, s| s.total_occupancy());
        let verd = self.style.verdant();
        Rect::at(l.0, l.1)
            .fill(verd.fill_color)
            .size(total.width(), total.height())
            .draw(surface);

        let mut n_ctx = self.ctx(Some(&children_current));
        for c in self.children.iter() {
            c.render(surface, &mut n_ctx, sender.clone());
        }
    }

    fn get_children(&self) -> Vec<Component> {
        self.children.clone()
    }

    fn get_style(&self) -> SenStyle {
        self.style.clone()
    }

    fn size(self: Rc<Self>, size: Size) -> Component {
        self.style.flume.style_with(move |_, style| {
            style.size = size;
        });

        self.clone()
    }

    fn color(self: Rc<Self>, color: Color) -> Component {
        self.style.verdant().fill(color);
        self.clone()
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

    fn click(self: Rc<Self>) {
        match self.event_on_click.lock() {
            Ok(v) => {
                if let Some(fun) = v.as_ref() {
                    fun();
                }
            },
            Err(e) => {
                if let Some(fun) = e.into_inner().as_ref() {
                    fun();
                }
            }
        }
    }
}

impl FlumeNode for Div {
    fn get_children_nodes(&self, n: usize) -> Option<Rc<dyn FlumeNode>> {
        match self.children.get(n) {
            Some(e) => Some(e.clone() as Rc<dyn FlumeNode>),
            None => None,
        }
    }

    fn ctx(&self, existing: Option<&PassCtx>) -> PassCtx {
        self.style.flume.ctx(existing)
    }

    fn get_flume(&self) -> FlumeElem {
        self.style.flume.clone()
    }
}

#[macro_export]
macro_rules! div {
    ( $($child:expr),* $(,)? ) => {
        Div::new(vec![
            $($child),*
        ])
    };
}
