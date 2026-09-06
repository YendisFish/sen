use std::sync::atomic::{AtomicUsize, Ordering};

use taffy::prelude::*;
use verdant::{render_surface::RenderSurface, text::{Span, TextStyle}, types::Color, vec::Vec2, window::WindowDraw};

use crate::{components::text::TEXT_MULTIPLIER, views::{Component, SenId, View}};

pub static LAST_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Layout {
    pub root: Option<NodeId>,
    pub taffy: TaffyTree,
}

impl Layout {
    pub fn new(size: Vec2) -> Layout {
        Layout {
            root: None,
            taffy: TaffyTree::new(),
        }
    }

    pub fn build_from(&mut self, win: &mut WindowDraw, c: &View, mut node: Option<NodeId>) {
        let taffy = &mut self.taffy;

        let style = c.get_style();
        let mut taffy_style = style.taffy().clone();

        if let Some(ctx) = c.get_ctx() {
            let span = Span::new(ctx.text.clone(), ctx.font, TextStyle {
                size: ctx.text_size,
                color: ctx.default_color,
                ..Default::default()
            });

            let num_lines: usize = ctx.text.lines().count();

            let (w, _) = win.rich_text_size(&[span.clone()]).into();
            taffy_style.size = Size {
                width: length(w),
                height: length((ctx.text_size * TEXT_MULTIPLIER) * num_lines as f32),
            };

            taffy_style.box_sizing = taffy::style::BoxSizing::ContentBox;
        }

        let new_node = match taffy.new_leaf(taffy_style) {
            Ok(n) => n,
            Err(_) => return,
        };

        c.set_id(SenId{
            sen: LAST_ID.fetch_add(1, Ordering::Relaxed),
            taffy: new_node
        });

        if let Some(n) = node {
            _ = taffy.add_child(n, new_node);
        } else {
            self.root = Some(new_node);
        }

        node = Some(new_node);

        let Some(comp_tree) = c.get_inner() else { return; };
        for com in comp_tree.as_ref() {
            self.build_from(win, com, node);
        }
    }
}
