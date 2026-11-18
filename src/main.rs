#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod diff;
mod theme;

use std::borrow::Cow;

use app::DiffApp;
use gpui_kit::*;

fn load_fonts(cx: &mut App) {
    cx.text_system()
        .add_fonts(vec![
            Cow::Borrowed(include_bytes!("../assets/Inter-Regular.ttf").as_slice()),
            Cow::Borrowed(include_bytes!("../assets/JetBrainsMono-Regular.ttf").as_slice()),
            Cow::Borrowed(include_bytes!("../assets/IBMPlexSans-Regular.ttf").as_slice()),
        ])
        .expect("failed to load application fonts");
}

fn launch(cx: &mut App) {
    gpui_kit::init(cx);
    load_fonts(cx);
    theme::apply(cx);

    #[cfg(not(target_family = "wasm"))]
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(1180.), px(820.)),
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("Text Diff".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    #[cfg(target_family = "wasm")]
    let options = WindowOptions::default();

    gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| DiffApp::new(window, cx))
    })
    .expect("failed to open window");

    cx.activate(true);
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    gpui_kit::application().run(launch);
}

#[cfg(target_family = "wasm")]
fn main() {
    use std::cell::RefCell;

    use gpui_kit::web::{WebBackendPreference, WebPlatform};

    thread_local! {
        static APPLICATION: RefCell<Option<ApplicationHandle>> = const { RefCell::new(None) };
    }

    gpui_kit::platform::web_init();

    let platform = std::rc::Rc::new(WebPlatform::new_with_backend(
        false,
        WebBackendPreference::Auto,
    ));
    let app = Application::with_platform(platform);
    let handle = app.run_embedded(launch);

    APPLICATION.with(|application| {
        *application.borrow_mut() = Some(handle);
    });
}
