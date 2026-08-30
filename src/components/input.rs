use std::sync::{Arc, Mutex, mpsc::Sender};

use taffy::TaffyTree;
use verdant::{text::Font, types::Color, vec::Vec2, window::WindowDraw};

use crate::{components::{bind::State, div::Div, many::Many, text::Text}, many, styling::SenStyle, views::{Component, Events, Id, SenId, Stylable, View}};

fn default_style() -> SenStyle {
    Default::default()
}

pub struct Input {
    id: Mutex<Option<SenId>>,
    data: Arc<State<String>>,
    style: Mutex<SenStyle>,
    font: Font,
}

impl Input {
    pub fn new(font: Font, state: Option<Arc<State<String>>>) -> Self {
        let m_font = font.clone();

        let ret = Self {
            id: Mutex::new(None),
            data: state.unwrap_or(State::new(String::new())),
            style: Mutex::new(default_style()),
            font: font,
        };

        ret.data.with(move |dat| {
            Div::new(
                Text::new(dat.clone(), m_font.clone())
            )
            .color(Color::WHITE)
            .as_view()
        });

        ret
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
    fn click(&self, set_focus: &mut bool) -> bool { false }
    fn drag_over(&self, view: Option<View>) {}

    fn key_down(&self, key: verdant::prelude::Key) {}
}

impl Component for Input {
    fn as_view(self) -> View {
        Arc::new(self)
    }

    fn get_ctx(&self) -> Option<crate::views::RenderCtx> { None }

    fn get_inner(&self) -> Option<Arc<Vec<View>>> {
        None
    }

    fn get_style(&self) -> SenStyle {
        todo!()
    }

    fn render(&self, window: &mut WindowDraw, tree: &mut TaffyTree, offset: Vec2, chan: Arc<Sender<View>>) {

    }
}
