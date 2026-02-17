use gpui::{AppContext, Application, WindowDecorations, WindowOptions};
use gpui_component::TitleBar;
use gpui_component::{Theme, ThemeMode, ThemeRegistry};
use std::path::PathBuf;

use super::assets::CombinedAssets;
use super::frame_root::FrameRoot;
use super::workspace::GpuiDockWorkspace;

pub fn run() {
    let app = Application::new().with_assets(CombinedAssets);

    app.run(|cx| {
        gpui_component::init(cx);
        if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, |_cx| {}) {
            eprintln!("Failed to watch themes directory: {err}");
        }
        let window_options = WindowOptions {
            titlebar: Some(TitleBar::title_bar_options()),
            window_decorations: Some(WindowDecorations::Client),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                window.set_window_title("rshogi GPUI P1");
                let view = cx.new(|cx| GpuiDockWorkspace::new(window, cx));
                Theme::change(ThemeMode::Dark, Some(window), cx);
                cx.new(|_| FrameRoot::new(view))
            })?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
