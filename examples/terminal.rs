use std::{io::{BufRead, BufReader, Error}, process::{self, Command}, sync::Arc, time::Duration};

use sen::{components::{bind::State, div::Div, input::Input, many::{self, Many}, text::Text}, layout::Layout, styling::{DisplayType, Margin, Padding}, ui::SenWindow, views::{Component, Stylable, View}, *};
use verdant::{vec, window};

fn main() -> Result<(), Error> {
    let mut renderer = Renderer::new().unwrap();
    let window = renderer.create_window("Sen Terminal", 1920, 1080);
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
    let fnt = font.clone();
    let fnt2 = font.clone();
    let prev: Arc<State<String>> = State::new(String::from(""));
    let after: Arc<State<String>> = State::new(String::from(""));

    let main_header = TerminalHeader(TerminalHeaderCtx { head: prev.clone(), foot: after.clone(), font: font.clone() });

    Div::new(many![
        prev.with(move |val| {
            Text::new(val.clone(), fnt.clone()).as_view()
        }),
        main_header.clone(),
        after.with(move |val| {
            Text::new(val.clone(), fnt2.clone()).as_view()
        })
    ].display(DisplayType::Flex(FlexDirection::Column))).color(Color::WHITE).size(Vec2::new(1920., 1080.))
}

struct TerminalHeaderCtx {
    head: Arc<State<String>>,
    foot: Arc<State<String>>,
    font: Font,
}

#[allow(non_snake_case)]
fn TerminalHeader(ctx: TerminalHeaderCtx) -> Arc<State<TerminalHeaderCtx>> {
    let thead = State::new(ctx);

    thead.with(|ctx| {
        if ctx.foot.get().len() > 0 {
            ().as_view()
        } else {
            let hd = ctx.head.clone();
            let ft = ctx.foot.clone();
            let fot = ctx.font.clone();
            many![
                Text::new(":>".into(), fot.clone()),
                Input::new(fot.clone(), None).on_submit(move |st| {
                    let cur_str = st.get().clone();

                    let mut old_vec = hd.clone().get().clone();
                    old_vec = old_vec + cur_str.as_str() + "\n";

                    _ = hd.clone().set(old_vec);
                    _ = st.set("".into());
                })
            ].as_view()
        }
    })
}

fn run_command(outp: Arc<State<String>>) {
    std::thread::spawn(move || {
        let mut cmd = Command::new("ls").spawn().expect("death");
        let stdout = cmd.stdout.take().expect("death 2");

        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            let txt = outp.get().clone();
            _ =  outp.set(txt + line.expect("death 4").as_str());
            std::thread::sleep(Duration::from_millis(300));
        }

        let status = cmd.wait().expect("death 3");
    });
}
