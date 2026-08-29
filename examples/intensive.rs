use std::{hash::Hash, io::Error, sync::Arc, time::Duration};

use sen::{components::{bind::State, div::Div, many::Many}, layout::Layout, styling::DisplayType, ui::SenWindow, views::{Component, Stylable, View}, *};
use verdant::{vec, window};

fn main() -> Result<(), Error> {
    let mut renderer = Renderer::new().unwrap();
    let window = renderer.create_window("Counter", 1920, 1080);
    let mut swin = SenWindow::new(window, Vec2::new(1920., 1080.));
    swin.force_thread_timeout = Some(Duration::from_secs(1) / 60);

    swin.start(&mut renderer, &mut App().as_view());

    Ok(())
}

/*
 * Interestingly the components for Sen only use
 * 44MB until they are handed over to the WGPU
 * instance. Seems like loading 500,000 components
 * for the GPU shouldn't happen.
 */

#[allow(non_snake_case)]
pub fn App() -> impl Component {
    let mut vec: Vec<View> = Vec::new();

    for _ in 0..500000 {
        vec.push(
            Div::new(()).as_view()
        );
    }

    Many::new(vec)
}
