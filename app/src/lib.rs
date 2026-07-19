// Declares scene_viewport.rs as part of this crate — without this line,
// the file exists on disk but the compiler doesn't know about it.
mod scene_viewport;
mod url_listener;

use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::WindowPlugin;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, EguiTextureHandle, egui};
use bevy_fontmesh::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use scene_viewport::SceneTexture;
use url_listener::*;

/// Entry point called by main.rs, identically for both native and wasm builds.
/// Bevy's `winit` backend handles the native/wasm distinction internally —
/// on wasm32 it attaches to the <canvas> named below instead of opening a
/// native OS window, so no #[cfg(target_arch = "wasm32")] split is needed here.
pub fn run(initial_hash: Option<String>) {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // Explicitly bind to the canvas in index.html by CSS selector,
                        // rather than relying on Bevy's default canvas-detection
                        // behavior on wasm (which can vary and is easy to get wrong).
                        // Ignored entirely on native builds.
                        canvas: Some("#bevy".to_string()),
                        // Makes the canvas resize to fill its parent element in the
                        // page, rather than staying a fixed pixel size — matters for
                        // a responsive page layout. Ignored on native.
                        fit_canvas_to_parent: true,
                        // Stops the browser from intercepting certain events (e.g.
                        // right-click context menu) so they don't fight with in-app
                        // camera controls / picking. Ignored on native.
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                // no meta check (404s)
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
        // Debug-only world inspector, toggled with Escape — harmless to leave
        // in wasm builds too, just an extra egui window the user can open.
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugins(scene_builder::LightsPlugin)
        .add_plugins(scene_builder::LoadingPlugin)
        .add_plugins(scene_builder::ParseDrawPlugin)
        .add_plugins(scene_builder::SkyboxFlytoCameraPlugin)
        .add_plugins(UrlListenerPlugin { initial_hash })
        .add_systems(Update, debug_print_hash)
        // Spawns the off-screen render target + camera that the 3D scene
        // draws into, so it can be displayed inside an egui image widget
        // rather than directly to the window/canvas.
        .add_systems(Startup, scene_viewport::setup_render_target)
        .add_systems(EguiPrimaryContextPass, draw_ui)
        .run();
}

// One unavoidable top-level `.show(ctx, ...)` call below is still flagged
// deprecated by egui (which wants show_inside for everything nested) — but
// something has to be the outermost panel, so it's allowed here explicitly
// rather than silenced project-wide. This is unrelated to the WindowPlugin
// change above and still applies.
#[allow(deprecated)]
fn draw_ui(mut contexts: EguiContexts, mut scene_texture: ResMut<SceneTexture>) -> Result {
    // Register the Bevy render-target image with egui once, on the first
    // frame, and remember the resulting texture id for subsequent frames.
    if scene_texture.egui_texture_id.is_none() {
        let id = contexts.add_image(EguiTextureHandle::Strong(
            scene_texture.image_handle.clone(),
        ));
        scene_texture.egui_texture_id = Some(id);
    }

    let ctx = contexts.ctx_mut()?;

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::Panel::left("editing_panel").show_inside(ui, |ui| {
            ui.heading("Class editor");
            // ui_panels::class_editor(ui, /* ... */); // TODO: wire in ui_panels widgets
        });

        // Remaining central area displays the embedded 3D scene.
        // `&*scene_texture` derefs the ResMut wrapper down to &SceneTexture,
        // which is what scene_viewport::show expects.
        scene_viewport::show(ui, &scene_texture);
    });

    Ok(())
}

fn debug_print_hash(url_hash: Res<UrlHash>) {
    if url_hash.is_changed() {
        println!("{:#?}", *url_hash);
    }
}
