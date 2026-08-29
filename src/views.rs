use std::sync::{Arc, Mutex, mpsc::Sender};
use taffy::{AlignSelf, FlexDirection, NodeId, Style, TaffyTree};
use verdant::{event::Key, text::Font, types::Color, vec::Vec2, window::WindowDraw};

use crate::{layout, styling::{DisplayType, Margin, Padding, SenStyle}, ui::SenWindow};

pub type View = Arc<dyn Component>;

pub struct RenderCtx {
    pub text: String,
    pub text_size: f32,
    pub font: Font,
    pub default_color: Color,
}

#[derive(Clone, Copy)]
pub(crate) struct SenId {
    pub sen: usize,
    pub taffy: NodeId,
}

// styling trait
pub trait Stylable {
    fn size(self, size: Vec2) -> Self;
    fn color(self, color: Color) -> Self;
    fn display(self, tp: DisplayType) -> Self;
    fn align_self(self, slf: AlignSelf) -> Self;
    fn padding(self, pad: Padding) -> Self;
    fn margin(self, mar: Margin) -> Self;
    fn outline(self, color: Color, size: f32) -> Self;
    fn rounding(self, rounding: f32) -> Self;
}

pub(crate) trait Id {
    fn set_id(&self, id: SenId);
    fn get_id(&self) -> Option<SenId>;
}

/*
 * In the case that a component cannot return an Id
 * then its children will inherite the bounds and
 * genealogy from the next available parent!!!
 *
 * With taffy I might just be able to skip the vec
 * and pass the parent down into the children during
 * style passing.
 */

impl Id for () {
    fn set_id(&self, id: SenId) {}
    fn get_id(&self) -> Option<SenId> { None }
}

pub(crate) trait Events {
    fn click(&self, request_focus: &mut bool) -> bool;
    fn drag_over(&self, view: Option<View>);
    fn key_down(&self, key: Key);
}

impl Events for () {
    fn click(&self, set_focus: &mut bool) -> bool { false }
    fn drag_over(&self, view: Option<View>) {}
    fn key_down(&self, key: Key) { }
}

// this comment makes things more readable :)
pub trait Component: Id + Events {
    fn as_view(self) -> View;
    fn get_inner(&self) -> Option<Arc<Vec<View>>>;
    fn render(&self, window: &mut WindowDraw, tree: &mut TaffyTree, offset: Vec2, chan: Arc<Sender<View>>);

    fn get_style(&self) -> SenStyle;
    fn get_ctx(&self) -> Option<RenderCtx> {
        None
    }
}

impl Component for () {
    fn as_view(self) -> View {
        Arc::new(())
    }

    fn get_inner(&self) -> Option<Arc<Vec<View>>> {
        None
    }

    fn render(&self, window: &mut WindowDraw, tree: &mut TaffyTree, offset: Vec2, chan: Arc<Sender<View>>) { }
    fn get_style(&self) -> SenStyle {
        Default::default()
    }
}
