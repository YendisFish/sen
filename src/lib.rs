use std::{collections::HashMap, rc::Rc, sync::{Arc, OnceLock, mpsc::{Receiver, Sender, channel}}, thread::current};

use flume::{FlumeElem, FlumeNode, PassCtx, Size, Space, compute};
use verdant::{Renderer, canvas::Canvas, event::WindowEvent, render_surface::RenderSurface, vec::Vec2, window::{WindowDraw, WindowId}};

use crate::{components::state, view::Component};

pub mod view;
pub mod components;
pub mod style;

pub struct Sen {
    window: WindowId,
    window_size: Size,
    pub channel: SenChannel,
    pub(crate) layout_arena: HashMap<usize, SenNode>,
    pub(crate) last_id: usize,
    pub(crate) root_node: Option<usize>,
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
            root_node: None,
        }
    }

    pub fn start(&mut self, renderer: &mut Renderer, root: Component) {
        self.compose_layout(root.clone(), None, None, None, false);

        if let Some(root_id) = root.clone().get_id() {
            self.root_node = Some(root_id);
        }

        let mut init: bool = true;
        while renderer.is_running() {
            for (id, event) in renderer.poll() {
                match event {
                    WindowEvent::CloseRequested => {
                        renderer.close_window(id);
                    },
                    WindowEvent::PointerButton { pressed: false, position, .. } => {
                        let Some(root_id) = self.root_node else { continue; };
                        let Some(hit) = self.hit_node(position, root_id) else { continue; };
                        hit.click();
                    },
                    _ => {},
                }
            }

            if let Some(mut win) = renderer.get_window(self.window) {
                while let Some(msg) = self.channel.reciever.try_recv().ok() {
                    let Some(elem) = self.layout_arena.get(&msg.target_id) else { continue; };
                    if let Some(stateful) = elem.clone().target.is_stateful() {
                        let new_children = stateful.generate_children();
                        stateful.set_children(new_children);
                    }

                    self.mark_branch_dirty(msg.target_id);
                    self.remove_branch(msg.target_id, false);
                    self.compose_layout(root.clone(), None, None, None, false);

                    let mut ctx = PassCtx::from_root(root.as_ref());
                    root.render(&mut win, &mut ctx, self.channel.sender.clone());
                }

                if init {
                    let mut ctx = PassCtx::from_root(root.as_ref());
                    root.render(&mut win, &mut ctx, self.channel.sender.clone());

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
                self.layout_arena.insert(self.last_id, SenNode::new(&component.get_flume(), parent_id, component.clone()));

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

    fn remove_branch(&mut self, node_id: usize, remove_anchor: bool) {
        if let Some(node) = self.layout_arena.remove(&node_id) {
            for chil in node.children.iter() {
                self.remove_branch(*chil, true);
            }

            if !remove_anchor {
                self.layout_arena.insert(node_id, node);
            }
        }
    }

    fn hit_node(&self, position: Vec2, from: usize) -> Option<Component> {
        let Some(nd) = self.layout_arena.get(&from) else { return None; };
        let Some((pos, size_x, size_y)) = nd.style.style_with(|_, style| {
            let Some(p) = style.cached_pos else { return None; };
            Some((p, style.size.width(), style.size.height()))
        }) else {
            return None
        };

        let in_x = position.x > pos.0 && position.x < pos.0 + size_x;
        let in_y = position.y > pos.1 && position.y < pos.1 + size_y;
        if !(in_x && in_y) {
            return None;
        }

        let mut current = from;
        'upper: while let Some(node) = self.layout_arena.get(&current) {
            if node.target.get_children().len() <= 0 {
                return Some(node.target.clone());
            }

            for child in node.target.get_children() {
                let Some((pos, size_x, size_y)) = child.get_flume().style_with(|_, style| {
                    let Some(p) = style.cached_pos else { return None; };
                    Some((p, style.size.width(), style.size.height()))
                }) else {
                    return None
                };

                let in_x = position.x > pos.0 && position.x < pos.0 + size_x;
                let in_y = position.y > pos.1 && position.y < pos.1 + size_y;
                if !(in_x && in_y) {
                    continue;
                }

                let Some(id) = child.get_id() else { return None; };
                current = id;
                continue 'upper;
            }

            break;
        }

        return Some(nd.target.clone());
    }
}

pub struct SenChannel {
    pub sender: Arc<Sender<SenNotif>>,
    pub(crate) reciever: Receiver<SenNotif>,
}

pub struct SenNotif {
    pub target_id: usize,
}

#[derive(Clone)]
pub struct SenNode {
    parent: Option<usize>,
    style: FlumeElem,
    children: Vec<usize>,
    target: Component,
}

impl SenNode {
    pub fn new(stl: &FlumeElem, parent: Option<usize>, target: Component) -> Self {
        Self {
            parent: parent,
            style: stl.clone(),
            children: vec![],
            target: target.clone(),
        }
    }
}
