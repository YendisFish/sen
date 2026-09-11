use std::{io::Error, rc::Rc};

use flume::Size;
use sen::{Sen, components::div::Div, div, view::{Component, View}};
use verdant::{Renderer, types::Color};

fn main() -> Result<(), Error> {
    let mut renderer = Renderer::new().unwrap();
    let window = renderer.create_window("Sen Counter", 1920, 1080);
    let sen = Sen::new(window, Size::take(1920., 1080.));

    sen.start(&mut renderer, App());

    Ok(())
}

#[allow(non_snake_case)]
pub fn App() -> Component {
    div![
        div!().size(Size::take(50., 200.)).color(Color::BLUE),
        div!().size(Size::take(50., 200.)).color(Color::GREEN),
    ].size(Size::take(200., 200.))
}
