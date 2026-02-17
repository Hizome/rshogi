#[cfg(feature = "ui-egui")]
mod app;
mod core;
#[cfg(feature = "ui-egui")]
mod ui;
#[cfg(feature = "ui-gpui")]
mod ui_gpui;

use shogi::bitboard::Factory as BBFactory;

fn main() {
    BBFactory::init();

    #[cfg(all(feature = "ui-egui", not(feature = "ui-gpui")))]
    {
        run_egui();
        return;
    }

    #[cfg(all(feature = "ui-gpui", not(feature = "ui-egui")))]
    {
        ui_gpui::run();
        return;
    }

    #[cfg(all(feature = "ui-egui", feature = "ui-gpui"))]
    {
        eprintln!("both ui features are enabled; starting gpui by default");
        ui_gpui::run();
        return;
    }

    #[cfg(not(any(feature = "ui-egui", feature = "ui-gpui")))]
    {
        eprintln!("no ui feature enabled. use --features ui-egui or --features ui-gpui");
    }
}

#[cfg(feature = "ui-egui")]
fn run_egui() {
    use app::state::RShogiApp;

    let options = eframe::NativeOptions::default();
    if let Err(err) = eframe::run_native(
        "rshogi P1",
        options,
        Box::new(|_cc| Ok(Box::new(RShogiApp::new()))),
    ) {
        eprintln!("failed to start app: {err}");
    }
}
