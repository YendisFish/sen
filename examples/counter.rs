use std::{hash::Hash, io::Error, sync::Arc, time::Duration};

use sen::{components::{bind::State, div::Div, input::Input, many::Many}, layout::Layout, styling::{DisplayType, Margin, Padding}, ui::SenWindow, views::{Component, Stylable, View}, *};
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
    let font_clone = font.clone();
    let s = State::new(0);
    Div::new(many![
        s.with(move |val| {
            sen::components::text::Text::new(val.to_string(), font.clone())
                .text_size(25.)
                .padding(Padding::Top { t: 20. })
                .as_view()
        }),
        Div::new(
            Button("Click Me!", enclose!([s] move || {
                let val = *s.get();
                _ = s.set(val + 1);
            }))
        ).margin(Margin::Top { t: 10. }).color(Color::TRANSPARENT),
    ].display(DisplayType::Flex(FlexDirection::Column))).color(Color::WHITE).size(Vec2::new(1920., 1080.))
}

#[allow(non_snake_case)]
fn Button(text: impl Into<String>, fun: impl Fn() + 'static + Send + Sync) -> Arc<State<(String, Color)>> {
    let font = Font::load("/System/Library/Fonts/SFNS.ttf").unwrap();
    let state = State::new((text.into(), Color::WHITE));

    let f = Arc::new(fun);
    state.with(move |val| {
        let fclone = f.clone();
        Div::new(many![
            sen::components::text::Text::new(val.0.clone(), font.clone())
                .text_size(25.)
                .padding(Padding::uniform(10.))
        ])
        .on_click(move || fclone.as_ref()())
        .color(Color::AQUAMARINE)
        .align_self(AlignSelf::FLEX_START)
        .rounding(15.)
        .outline(Color::BLACK, 5.)
        .as_view()
    })
}
