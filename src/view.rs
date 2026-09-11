use std::{rc::Rc};

use flume::{Direction, FlumeElem, FlumeNode, FlumeStyle, FlumeWrap, PassCtx, Size, Space};
use verdant::{render_surface::RenderSurface, types::Color, window::WindowDraw};

use crate::style::SenStyle;

pub type Component = Rc<dyn View>;
pub trait View: FlumeNode {
    fn render(&self, surface: &mut WindowDraw, ctx: &mut PassCtx);
    fn get_children(&self) -> Vec<Component>;

    // styling
    fn get_style(&self) -> SenStyle;
    fn size(self: Rc<Self>, size: Size) -> Component;
    fn color(self: Rc<Self>, color: Color) -> Component;

    //id
    fn get_id(self: Rc<Self>) -> Option<usize>;
    fn set_id(self: Rc<Self>, id: usize);
}

impl View for () {
    fn render(&self, surface: &mut WindowDraw, ctx: &mut PassCtx) { }
    fn get_children(&self) -> Vec<Component> {
        vec![]
    }

    fn get_style(&self) -> SenStyle {
        SenStyle::new()
    }

    fn size(self: Rc<Self>, _: Size) -> Component {
        self.clone()
    }

    fn color(self: Rc<Self>, color: Color) -> Component {
       self.clone()
    }

    fn get_id(self: Rc<Self>) -> Option<usize> {
        None
    }

    fn set_id(self: Rc<Self>, id: usize) { }
}
