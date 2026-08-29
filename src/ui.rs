use std::{any::Any, collections::{BinaryHeap, HashMap}, process::exit, sync::{Arc, mpsc::{Receiver, Sender, channel}}, time::{self, Duration, Instant}};

use taffy::{NodeId, Size, TaffyTree, style_helpers::length};
use verdant::{prelude::*, window::Window};

use crate::{components::{div::Div, many::Many}, layout::Layout, ui::UiError::CastError, views::{Component, Stylable, View}};

pub enum UiError {
    CastError,
    StateSetError,
}

pub struct SenWindow {
    window: WindowId,
    sender: Arc<Sender<View>>,
    recv: Receiver<View>,
    layout: Layout,
    pub force_thread_timeout: Option<Duration>,
    pub(crate) ctx: WindowCtx,
    pub window_size: Vec2,
}

impl SenWindow {
    pub fn new(window: WindowId, size: Vec2) -> Self {
        let chan: (Sender<View>, Receiver<View>) = channel();
        Self {
            window: window,
            sender: Arc::new(chan.0),
            recv: chan.1,
            layout: Layout::new(Vec2::new(1920., 1080.)),
            force_thread_timeout: None,
            ctx: Default::default(),
            window_size: size,
        }
    }

    pub fn start(&mut self, renderer: &mut Renderer, view: &mut View) {
        let app = Div::new(Many::new(vec![view.clone()])).size(self.window_size).as_view();

        let mut first = false;
        while renderer.is_running() {
            let elapsed = Instant::now();

            for (id, event) in renderer.poll() {
                match event {
                    WindowEvent::CloseRequested => { renderer.close_window(id); },
                    WindowEvent::PointerButton { pressed: true, button, position, .. } => {
                        self.ctx.mouse_left_down = true;

                        let mut vec: Vec<(View, usize)> = Vec::new();
                        find_views_at(position, &app, Vec2::new(0., 0.), &mut self.layout.taffy, &mut vec, 0);

                        vec.sort_unstable_by_key(|&(_, depth)| std::cmp::Reverse(depth));
                        for (vw, _) in vec.iter() {
                            if vw.click() {
                                break;
                            }
                        }
                    },
                    WindowEvent::PointerButton { pressed: false, button, position, .. } => {
                        if self.ctx.dragging {
                            // trigger drag_drop

                            // dragged component could possibly be no longer on the view tree?
                        }

                        //mouse up
                        self.ctx.mouse_left_down = false;
                        self.ctx.dragging = false;
                        self.ctx.in_cursor = None;
                    },
                    WindowEvent::PointerMoved { device_id, position, source } => {
                        if self.ctx.mouse_left_down {
                            self.ctx.dragging = true;

                            // bring dragged view with us!
                            let mut vec: Vec<(View, usize)> = Vec::new();
                            find_views_at(position, &app, Vec2::new(0., 0.), &mut self.layout.taffy, &mut vec, 0);

                            match vec.iter().min_by_key(|t| t.1) {
                                Some(l) => self.ctx.in_cursor = Some(l.0.clone()),
                                None => break,
                            };
                        }

                        let mut vec: Vec<(View, usize)> = Vec::new();
                        find_views_at(position, &app, Vec2::new(0., 0.), &mut self.layout.taffy, &mut vec, 0);

                        if self.ctx.dragging {
                            // trigger drag_over on all elements!
                            for c in vec.iter() {
                                c.0.drag_over(match &self.ctx.in_cursor {
                                    Some(elem) => Some(elem.clone()),
                                    None => None,
                                });
                            }
                        } else {
                            // trigger mouse_over
                        }
                    },
                    _ => {},
                }
            }

            if let Some(mut win) = renderer.get_window(self.window) {
                if let Some(v) = self.recv.try_recv().ok() {
                    win.background(Color::BLACK);
                    if let Some(id) = v.get_id() {
                        _ = self.layout.taffy.set_children(id.taffy, &[]);

                        _ = self.layout.taffy.set_style(id.taffy, v.get_style().taffy);
                        let Some(layout) = self.layout.taffy.layout(id.taffy).ok() else { continue; };

                        match v.get_inner() {
                            Some(chil) => {
                                for c in chil.as_ref() {
                                    self.layout.build_from(
                                        &mut win,
                                        c,
                                        Some(id.taffy));
                                }
                            },
                            None => {},
                        }

                        _ = self.layout.taffy.mark_dirty(id.taffy);

                        let Some(root_id) = app.get_id() else { continue; };
                        _ = self.layout.taffy.compute_layout(root_id.taffy, Size {
                            height: length(win.get_height()),
                            width: length(win.get_width()),
                        });
                    }

                    app.render(&mut win, &mut self.layout.taffy, Vec2::ZERO, self.sender.clone());
                }

                if !first {
                    self.layout.build_from(&mut win, &app, None);

                    _ = self.layout.taffy.compute_layout(app.get_id().unwrap().taffy, Size {
                        height: length(1920.),
                        width: length(1080.),
                    });

                    app.render(&mut win, &mut self.layout.taffy, Vec2::new(0., 0.), self.sender.clone());
                    first = true;
                }
            }

            _ = renderer.flush().unwrap();

            if let Some(timeout) = self.force_thread_timeout {
                let frame_time = elapsed.elapsed();
                if frame_time < timeout {
                    std::thread::sleep(timeout - frame_time);
                }
            }
        }
    }
}

fn find_views_at(
    position: Vec2,
    vw: &View,
    offset: Vec2,
    tree: &mut TaffyTree,
    vec: &mut Vec<(View, usize)>,
    last_depth: usize,
) {
    let Some(id) = vw.get_id() else { return; };
    let Some(layout) = tree.layout(id.taffy).ok() else { return; };

    let x = offset.x + layout.location.x;
    let y = offset.y + layout.location.y;

    let in_x = position.x >= x && position.x <= x + layout.size.width;
    let in_y = position.y >= y && position.y <= y + layout.size.height;
    if !(in_x && in_y) {
        return;
    }

    let new_offset = Vec2::new(x, y);
    if let Some(children) = vw.get_inner() {
        for c in children.as_ref() {
            find_views_at(position, c, new_offset, tree, vec, last_depth + 2);
        }
    }

    vec.push((vw.clone(), last_depth + 1));
}

pub(crate) struct WindowCtx {
    pub(crate) mouse_left_down: bool,
    pub(crate) dragging: bool,
    pub(crate) in_cursor: Option<View>,
}

impl Default for WindowCtx {
    fn default() -> Self {
        WindowCtx {
            mouse_left_down: false,
            dragging: false,
            in_cursor: None,
        }
    }
}
