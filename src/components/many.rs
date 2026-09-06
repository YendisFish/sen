use std::sync::{Arc, Mutex, mpsc::Sender};

use taffy::{AlignSelf, Display, TaffyTree};
use verdant::{vec::Vec2, window::WindowDraw};

use crate::{styling::{DisplayType::{self, Flex}, SenStyle}, views::{Component, Events, Id, SenId, Stylable, View}};


pub struct Many {
    id: Mutex<Option<SenId>>,
    views: Arc<Vec<View>>,
    pub(crate) style: Mutex<SenStyle>,
}

/*
 * Many is weird in that its a Stylable-like element
 * but doesn't actually implement the trait. I chose
 * this specifically because it's technically meant to
 * expose its children to the parent component and not
 * implement any styling downwards.
 *
 * The exception to this rule was that Many should have
 * options on the layout of components which it holds.
 * These are exposed to the user.
 */
impl Many {
    pub fn new(v: Vec<View>) -> Self {
        let mut tffy: taffy::Style<String> = taffy::Style::default();
        tffy.align_self = Some(AlignSelf::STRETCH);

        let mut r = SenStyle {
            verdant: Default::default(),
            taffy: Arc::new(Mutex::new(tffy)),
            ..Default::default()
        };

        Self {
            id: Mutex::new(None),
            views: Arc::new(v),
            style: Mutex::new(r),
        }
    }

    pub fn display(mut self, tp: DisplayType) -> Self {
        match self.style.lock() {
            Ok(val) => {
                match tp {
                    DisplayType::Flex(d) => {
                        val.taffy().display = Display::Flex;
                        val.taffy().flex_direction = d;
                    },
                    DisplayType::Grid() => {

                    },
                }
            },
            Err(e) => {
                let val = e.into_inner();
                match tp {
                    DisplayType::Flex(d) => {
                        val.taffy().display = Display::Flex;
                        val.taffy().flex_direction = d;
                    },
                    DisplayType::Grid() => {

                    },
                }
            }
        };


        self
    }
}

impl Id for Many {
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

impl Events for Many {
    fn click(&self, set_focus: &mut bool) -> bool { false }
    fn drag_over(&self, view: Option<View>) {}

    fn key_down(&self, key: verdant::prelude::Key) {}
}

impl Component for Many {
    fn as_view(self) -> View {
        Arc::new(self)
    }

    fn get_inner(&self) -> Option<Arc<Vec<View>>> {
        Some(self.views.clone())
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
        for c in self.views.as_ref() {
            c.render(window, tree, offset, chan.clone());
        }
    }
}

#[macro_export]
macro_rules! many {
    ( $($x:expr),* $(,)? ) => {
        Many::new(vec![
            $( $x.as_view(), )*
        ])
    };
}
