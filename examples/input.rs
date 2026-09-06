
use std::{hash::Hash, io::Error, sync::Arc, time::Duration};

use sen::{components::{bind::State, div::Div, input::Input, many::Many, text::Text}, layout::Layout, styling::{DisplayType, Margin, Padding}, ui::SenWindow, views::{Component, Stylable, View}, *};
use verdant::{vec, window};

fn main() -> Result<(), Error> {
    let mut renderer = Renderer::new().unwrap();
    let window = renderer.create_window("Counter", 1920, 1080);
    let mut swin = SenWindow::new(window, Vec2::new(1920., 1080.));

    #[cfg(target_os =  "macos")]
    {
        swin.force_thread_timeout = Some(Duration::from_secs(1) / 60);
    }

    swin.start(&mut renderer, &mut App().as_view());

    Ok(())
}

#[allow(non_snake_case)]
fn App() -> impl Component {
    let font = Font::load("/System/Library/Fonts/SFNS.ttf").unwrap();
    Div::new(many!(
        Input::new(font, None).size(Vec2::new(300., 50.))
    )).color(Color::WHITE).size(Vec2::new(1920., 1080.))
}
