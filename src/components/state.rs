use std::{rc::Rc, sync::{Arc, Mutex, OnceLock, mpsc::Sender}};

use flume::{FlumeElem, FlumeNode, PassCtx, Size, layout};
use verdant::{types::Color, window::WindowDraw};

use crate::{SenNotif, style::SenStyle, view::{Component, Stateful, View}};

pub struct StateHandle<T: Send + Sync + 'static> {
    owner: usize,
    hook: Arc<Mutex<T>>,
    sender: Arc<Sender<SenNotif>>,
}

impl<T: Send + Sync + 'static> StateHandle<T> {
    pub fn set(&self, data: T) {
        match self.hook.lock() {
            Ok(mut val) => {
                *val = data;
            },
            Err(e) => {
                let mut val = e.into_inner();
                *val = data;
            }
        }

        _ = self.sender.send(SenNotif {
            target_id: self.owner,
        });
    }
}

pub struct State<T: Send + Sync + 'static> {
    id: Mutex<Option<usize>>,
    style: SenStyle,
    hook: Arc<Mutex<T>>,
    children: Mutex<Vec<Component>>,
    pub(crate) generator: OnceLock<Box<dyn Fn(&T) -> Vec<Component> + 'static>>,
    channel: OnceLock<Arc<Sender<SenNotif>>>,
}

impl<T: Send + Sync + 'static> State<T> {
    pub fn new(val: T) -> Rc<Self> {
        Rc::new(Self {
            id: Mutex::new(None),
            style: SenStyle::sync(),
            hook: Arc::new(Mutex::new(val)),
            children: Mutex::new(vec![]),
            generator: OnceLock::new(),
            channel: OnceLock::new(),
        })
    }

    pub fn with(self: Rc<Self>, fun: impl Fn(&T) -> Vec<Component> + 'static) -> Component {
        _ = self.generator.set(Box::new(fun));

        match self.children.lock() {
            Ok(mut v) => {
                let Some(fun) = self.generator.get() else { return self.clone(); };
                let Some(val) = self.hook.lock().ok() else { return self.clone(); };
                *v = fun(&val);
            },
            Err(e) => {
                let mut v = e.into_inner();

                let Some(fun) = self.generator.get() else { return self.clone(); };
                let Some(val) = self.hook.lock().ok() else { return self.clone(); };
                *v = fun(&val);
            }
        }

        self.clone()
    }

    pub fn set(self: &Rc<Self>, data: T) {
        let Some(chan) = self.channel.get() else { return; };

        let id_opt = match self.id.lock() {
            Ok(i) => *i,
            Err(e) => *e.into_inner(),
        };
        let Some(id) = id_opt else { return; };

        match self.hook.lock() {
            Ok(mut val) => {
                *val = data;
            },
            Err(e) => {
                let mut val = e.into_inner();
                *val = data;
            }
        }

        _ = chan.send(SenNotif {
            target_id: id,
        });
    }

    pub fn as_handle(self: &Rc<Self>) -> Option<StateHandle<T>> {
        let id_opt = match self.id.lock() {
            Ok(i) => *i,
            Err(e) => *e.into_inner(),
        };

        let Some(id) = id_opt else { return None; };
        let Some(sender) = self.channel.get() else { return None; };

        Some(StateHandle {
            owner: id,
            hook: self.hook.clone(),
            sender: sender.clone(),
        })
    }
}

impl<T: Send + Sync + 'static> View for State<T> {
    fn render(&self, surface: &mut WindowDraw, ctx: &mut PassCtx) {
        let children_current = ctx.clone();
        layout(ctx, self);

        let mut n_ctx = self.ctx(Some(&children_current));

        for c in self.get_children().iter() {
            c.render(surface, &mut n_ctx);
        }
    }

    fn get_children(&self) -> Vec<Component> {
        match self.children.lock() {
            Ok(v) => v.clone(),
            Err(e) => e.into_inner().clone(),
        }
    }

    fn get_style(&self) -> SenStyle {
        self.style.clone()
    }

    fn color(self: Rc<Self>, color: Color) -> Component {
        self.style.verdant().fill_color = color;
        self.clone()
    }

    fn size(self: Rc<Self>, size: Size) -> Component {
        self.style.flume.style_with(move |_, stle| {
            stle.size = size;
        });

        self.clone()
    }

    fn get_id(self: Rc<Self>) -> Option<usize> {
        match self.id.lock() {
            Ok(v) => *v,
            Err(e) => *e.into_inner(),
        }
    }

    fn set_id(self: Rc<Self>, id: usize) {
        match self.id.lock() {
            Ok(mut v) => *v = Some(id),
            Err(e) => {
                let mut v = e.into_inner();
                *v = Some(id);
            },
        }
    }

    fn is_stateful(self: Rc<Self>) -> Option<Rc<dyn Stateful>> {
        return Some(self as Rc<dyn Stateful>);
    }
}

impl<T: Send + Sync + 'static> FlumeNode for State<T> {
    fn get_children_nodes(&self, n: usize) -> Option<Rc<dyn FlumeNode>> {
        let children_lock = match self.children.lock() {
            Ok(v) => v,
            Err(e) => e.into_inner(),
        };


        if let Some(ret) = children_lock.get(n) {
            return Some(ret.clone() as Rc<dyn FlumeNode>);
        }

        None
    }

    fn ctx(&self, existing: Option<&PassCtx>) -> PassCtx {
        self.style.flume.ctx(existing)
    }

    fn get_flume(&self) -> FlumeElem {
        self.style.flume.clone()
    }
}

impl<T: Send + Sync + 'static> Stateful for State<T> {
    fn generate_children(&self) -> Vec<Component> {
        let Some(val) = self.hook.lock().ok() else { return vec![]; };
        match self.generator.get() {
            Some(fun) => fun(&val),
            None => vec![],
        }
    }
}
