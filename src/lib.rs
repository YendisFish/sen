use std::{rc::Rc, sync::{Arc, OnceLock, mpsc::{Receiver, Sender, channel}}};

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
    pub(crate) layout_arena: Vec<FlumeElem>,
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
            layout_arena: Vec::new(),
        }
    }

    pub fn start(&self, renderer: &mut Renderer, root: Component) {
        let avail = Space::available(self.window_size.width(), self.window_size.height());
        match compute(root.as_ref(), Some(avail), true) {
            Ok(_) => {},
            // should one day return a result
            Err(e) => {},
        }

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
                let mut ctx = PassCtx::from_root(root.as_ref());
                root.render(&mut win, &mut ctx);
            }

            _ = renderer.flush();
        }
    }
}

pub struct SenChannel {
    pub sender: Arc<Sender<Component>>,
    pub(crate) reciever: Receiver<Component>,
}
