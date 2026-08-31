use std::{cell::RefCell, io::Error, ops::Deref, rc::Rc, sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock, RwLockReadGuard, atomic::{AtomicBool, Ordering}, mpsc::Sender}};

use taffy::prelude::*;
use verdant::{vec::Vec2, window::WindowDraw};

use crate::{layout::{LAST_ID, Layout}, styling::SenStyle, ui::{SenWindow, UiError}, views::{Component, Events, Id, SenId, View}};

pub struct State<T: 'static> {
    id: Mutex<Option<SenId>>,
    item: RwLock<T>,
    closure: OnceLock<Box<dyn Fn(&T) -> View + 'static>>,
    current_view: RwLock<Arc<Vec<View>>>,
    notifier: OnceLock<Arc<Sender<View>>>,
    on_drag_over: OnceLock<Box<dyn Fn(View)>>,
}

impl<T: 'static> State<T> {
    pub fn new(val: T) -> Arc<Self> {
        Arc::new(Self {
            id: Mutex::new(None),
            item: RwLock::new(val),
            closure: OnceLock::new(),
            current_view: RwLock::new(Arc::new(vec![])),
            notifier: OnceLock::new(),
            on_drag_over: OnceLock::new(),
        })
    }

    pub fn with(self: &Arc<Self>, closure: impl Fn(&T) -> View + 'static) -> Arc<Self> {
        _ = self.closure.set(Box::new(closure));

        if let Some(arc) = self.closure.get() {
            let current_value = &*self.get();
            let new_views = arc.as_ref()(current_value);

            let mut current_view = match self.current_view.write() {
                Ok(val) => val,
                Err(e) => e.into_inner(),
            };

            *current_view = Arc::new(vec![new_views]);
        }

        self.clone()
    }

    pub fn recompute(self: &Arc<Self>) {
        if let Some(arc) = self.closure.get() {
            let current_value = &*self.get();
            let new_views = arc.as_ref()(current_value);

            let mut current_view = match self.current_view.write() {
                Ok(val) => val,
                Err(e) => e.into_inner(),
            };

            *current_view = Arc::new(vec![new_views]);
        }
    }

    pub fn get(&self) -> RwLockReadGuard<'_, T> {
        match self.item.read() {
            Ok(guard) => guard,
            Err(err) => err.into_inner(),
        }
    }

    pub fn set(self: &Arc<Self>, value: T) -> Result<(), UiError> {
        if let Some(mut val) = self.item.write().ok() {
            *val = value;
        }

        if let Some(arc) = self.closure.get() {
            let current_value = &*self.get();
            let new_views = arc.as_ref()(current_value);

            let mut current_view = match self.current_view.write() {
                Ok(val) => val,
                Err(e) => e.into_inner(),
            };

            *current_view = Arc::new(vec![new_views]);
        }

        if let Some(not) = &self.notifier.get() {
            match not.send(self.clone().as_view()).map_err(|_| UiError::StateSetError) {
                Ok(_) => {},
                Err(e) => return Err(e),
            }

            Ok(())
        } else {
            Err(UiError::StateSetError)
        }
    }

    pub fn on_drag_over(self: Arc<Self>, fun: impl Fn(View) + 'static) -> Arc<Self> {
        _ = self.on_drag_over.set(Box::new(fun));
        self
    }
}

impl<T: 'static> Id for Arc<State<T>> {
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

impl<T: 'static> Component for Arc<State<T>> {
    fn as_view(self) -> View { Arc::new(self) }

    fn get_inner(&self) -> Option<Arc<Vec<View>>> {
        let vec = match self.current_view.read() {
            Ok(val) => val,
            Err(e) => e.into_inner()
        };

        Some(vec.clone())
    }

    fn render(&self, window: &mut WindowDraw, tree: &mut TaffyTree, mut offset: Vec2, chan: Arc<Sender<View>>) {
        _ = self.notifier.set(chan.clone());

        let Some(id) = self.get_id() else { return; };
        let layout = match tree.layout(id.taffy) {
            Ok(l) => l,
            Err(_) => return,
        };

        let x = offset.x + layout.location.x;
        let y = offset.y + layout.location.y;

        offset.x = x;
        offset.y = y;

        let Some(views) = &self.current_view.read().ok() else { return; };
        for v in views.as_ref() {
            v.render(window, tree, offset, chan.clone());
        }
    }

    fn get_style(&self) -> SenStyle {
        Default::default()
    }
}

impl<T: 'static> Events for Arc<State<T>> {
    fn click(&self, set_focused: &mut bool) -> bool { false }
    fn drag_over(&self, view: Option<View>) {
        let Some(c) = self.on_drag_over.get() else { return; };
        let Some(vw) = view else { return; };
        c(vw);
    }

    fn key_down(&self, key: verdant::prelude::Key) {

    }
}
