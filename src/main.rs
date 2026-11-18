#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod diff;
mod file_io;
mod theme;

use std::borrow::Cow;

use app::DiffApp;
use gpui_kit::*;

gpui_kit::assets::icon_assets!(AppAssets, [ArrowLeftRight, Trash]);

#[cfg(target_family = "wasm")]
use std::cell::RefCell;

#[cfg(target_family = "wasm")]
thread_local! {
    static APPLICATION: RefCell<Option<ApplicationHandle>> = const { RefCell::new(None) };
    static APP_VIEW: RefCell<Option<WeakEntity<DiffApp>>> = const { RefCell::new(None) };
}

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
    app::init(cx);
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

    let (_, view) = gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| DiffApp::new(window, cx))
    })
    .expect("failed to open window");

    #[cfg(target_family = "wasm")]
    APP_VIEW.with(|slot| *slot.borrow_mut() = Some(view.downgrade()));
    #[cfg(not(target_family = "wasm"))]
    let _ = view;

    cx.activate(true);
}

#[cfg(target_family = "wasm")]
fn with_app_view(update: impl FnOnce(&mut DiffApp, &mut Window, &mut Context<'_, DiffApp>)) {
    let view = APP_VIEW.with(|slot| slot.borrow().clone());
    let Some(view) = view else {
        return;
    };

    APPLICATION.with(|slot| {
        let slot = slot.borrow();
        let Some(application) = slot.as_ref() else {
            return;
        };

        let _ = application.update(|cx| {
            let _ = view.update_in(cx, update);
        });
    });
}

/// Called by `index.html` after a file is read as UTF-8 text.
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn load_file_text(pane: &str, name: &str, text: String) {
    let Some(target) = app::FileTarget::from_id(pane) else {
        let message = "Unknown editor pane for the selected file.".to_string();
        with_app_view(move |app, _, cx| app.set_file_error(message, cx));
        return;
    };

    let name = name.to_string();
    with_app_view(move |app, window, cx| {
        app.load_text_named(target, name, text, window, cx);
    });
}

/// Called by `index.html` when a file must be rejected before decoding.
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn report_file_error(message: &str) {
    let message = message.to_string();
    with_app_view(move |app, _, cx| app.set_file_error(message, cx));
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    gpui_kit::application()
        .with_assets(AppAssets)
        .run(launch);
}

#[cfg(target_family = "wasm")]
fn main() {
    use gpui_kit::web::{WebBackendPreference, WebPlatform};

    gpui_kit::platform::web_init();

    let platform = std::rc::Rc::new(WebPlatform::new_with_backend(
        false,
        WebBackendPreference::Auto,
    ));
    let app = Application::with_platform(platform).with_assets(AppAssets);
    let handle = app.run_embedded(launch);

    APPLICATION.with(|application| {
        *application.borrow_mut() = Some(handle);
    });
}
