use std::{rc::Rc, sync::{Arc, Mutex}};

use flume::{FlumeElem, FlumeNode, PassCtx, Size, layout};
use verdant::{render_surface::RenderSurface, shapes::{Drawable, Rect}, types::Color, vec, window::WindowDraw};

use crate::{style::SenStyle, view::{Component, View}};

fn default_style() -> SenStyle {
    let stl = SenStyle::new();
    stl.verdant().fill(Color::RED);

    stl.flume.style_with(|_, fun| {
        fun.size = Size::take(10., 10.);
    });

    stl
}

pub struct Div {
    style: SenStyle,
    children: Vec<Component>,
}

impl Div {
    pub fn new(c: Vec<Component>) -> Rc<Self> {
        Rc::new(Self {
            style: default_style(),
            children: c,
        })
    }
}

impl View for Div {
    fn render(&self, surface: &mut WindowDraw, ctx: &mut PassCtx) {
        let children_current = ctx.clone();
        let l = layout(ctx, self);

        let total = self.style.flume.style_with(|_, s| s.total_occupancy());
        let verd = self.style.verdant();
        Rect::at(l.0, l.1)
            .fill(verd.fill_color)
            .size(total.width(), total.height())
            .draw(surface);

        let mut n_ctx = self.ctx(Some(&children_current));
        for c in self.children.iter() {
            c.render(surface, &mut n_ctx);
        }
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
}

impl FlumeNode for Div {
    fn get_children_nodes(&self, n: usize) -> Option<&dyn FlumeNode> {
        match self.children.get(n) {
            Some(e) => Some(e.as_ref()),
            None => None::<&dyn FlumeNode>,
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
