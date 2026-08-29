use std::sync::atomic::{AtomicUsize, Ordering};

use taffy::prelude::*;
use verdant::vec::Vec2;

use crate::views::{Component, SenId, View};

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

    pub fn build_from(&mut self, c: &View, mut node: Option<NodeId>) {
        let taffy = &mut self.taffy;

        let style = c.get_style();
        let taffy_style = style.taffy;

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
            self.build_from(com, node);
        }
    }
}
