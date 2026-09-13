use std::{collections::HashMap, rc::Rc, sync::{Arc, OnceLock, mpsc::{Receiver, Sender, channel}}};

use flume::{FlumeElem, FlumeNode, PassCtx, Size, Space, compute};
use verdant::{Renderer, canvas::Canvas, event::WindowEvent, render_surface::RenderSurface, window::{WindowDraw, WindowId}};

use crate::view::Component;

pub mod view;
pub mod components;
pub mod style;

pub struct Sen {
    window: WindowId,
    window_size: Size,
    pub channel: SenChannel,
    pub(crate) layout_arena: HashMap<usize, SenNode>,
    pub(crate) last_id: usize,
}

impl Sen {
    pub fn new(window: WindowId, size: Size) -> Self {
        let chan = channel();
        Sen {
            window: window,
            window_size: size,
            channel: SenChannel {
                sender: Arc::new(chan.0),
                reciever: chan.1,
            },
            layout_arena: HashMap::new(),
            last_id: 0,
        }
    }

    pub fn start(&mut self, renderer: &mut Renderer, root: Component) {
        self.compose_layout(root.clone(), None, None, None, false);

        let mut init: bool = true;
        while renderer.is_running() {
            for (id, event) in renderer.poll() {
                match event {
                    WindowEvent::CloseRequested => {
                        renderer.close_window(id);
                    },
                    _ => {},
                }
            }

            if let Some(mut win) = renderer.get_window(self.window) {
                while let Some(msg) = self.channel.reciever.recv().ok() {
                    // get child from arena
                    // check if child is stateful
                    // if stateful remove all previous children from arena and regenerate component children
                    // if not stateful (also do this after regenerating stateful children) recompute layout
                }
                if init {
                    let mut ctx = PassCtx::from_root(root.as_ref());
                    root.render(&mut win, &mut ctx);

                    init = false;
                }
            }

            _ = renderer.flush();
        }
    }

    fn compose_layout(
        &mut self,
        component: Component,
        parent_id: Option<usize>,
        mut avail: Option<Space>,
        parent_vec: Option<&mut Vec<usize>>,
        compute_only: bool,
    ) -> bool {
        if parent_id == None {
            avail = Some(Space::available(self.window_size.width(), self.window_size.height()));
        }

        match avail {
            None => {
                avail = Some(Space::available(self.window_size.width(), self.window_size.height()));
            },
            _ => {},
        }

        let r = match compute(component.clone().as_ref(), avail, false, None) {
            Ok(v) => v,
            Err(_) => false,
        };

        if compute_only {
            return r;
        }

        let id = match component.clone().get_id() {
            Some(v) => v,
            None => {
                self.last_id += 1;
                self.layout_arena.insert(self.last_id, SenNode::new(&component.get_flume(), parent_id));

                component.clone().set_id(self.last_id);
                self.last_id
            },
        };

        if let Some(vec) = parent_vec {
            vec.push(id);
        }

        let mut ret = false;
        let mut vec = Vec::new();
        for chil in component.get_children() {
            ret |= self.compose_layout(chil, Some(id), avail, Some(&mut vec), false);
        }

        let Some(node) = self.layout_arena.get_mut(&id) else { return ret; };
        node.children = vec;

        ret
    }

    fn mark_branch_dirty(&mut self, node_id: usize) {
        let mut current_id = node_id;
        while let Some(node) = self.layout_arena.get(&current_id) {
            node.style.style_with(|_, stle| stle.is_dirty = true);

            if let Some(par) = node.parent {
                current_id = par;
                continue;
            }

            break;
        }
    }

    fn remove_branch(&mut self, node_id: usize) {
        if let Some(node) = self.layout_arena.remove(&node_id) {
            for chil in node.children.iter() {
                self.remove_branch(*chil);
            }
        }
    }
}

pub struct SenChannel {
    pub sender: Arc<Sender<Component>>,
    pub(crate) reciever: Receiver<Component>,
}

pub struct SenNotif {
    pub target_id: usize,
}

#[derive(Clone)]
pub struct SenNode {
    parent: Option<usize>,
    style: FlumeElem,
    children: Vec<usize>,
}

impl SenNode {
    pub fn new(stl: &FlumeElem, parent: Option<usize>) -> Self {
        Self {
            parent: parent,
            style: stl.clone(),
            children: vec![],
        }
    }
}
