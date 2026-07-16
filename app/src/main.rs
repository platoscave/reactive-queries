// Bevy's `winit` backend handles the native/wasm distinction internally —
// on wasm32, it automatically attaches to a <canvas> in the page rather than
// opening a native OS window. Unlike ui_panels (a pure eframe app, which
// needs an explicit eframe::WebRunner wasm entry point), app is a
// Bevy-hosted process with egui layered on top via bevy_egui, so both
// native and wasm builds just call the same run() function — no
// #[cfg(target_arch = "wasm32")] split needed here.
fn main() {
    app::run();
}
