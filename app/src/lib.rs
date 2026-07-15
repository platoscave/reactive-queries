mod scene_viewport;

use crate::scene_viewport::SceneTexture;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, EguiTextureHandle, egui};
use bevy_fontmesh::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        // Lights
        .add_plugins(scene_builder::LightsPlugin)
        .add_plugins(scene_builder::LoadingPlugin)
        .add_plugins(scene_builder::ParseDrawPlugin)
        .add_plugins(scene_builder::SkyboxFlytoCameraPlugin)
        .add_systems(Startup, scene_viewport::setup_render_target)
        .add_systems(EguiPrimaryContextPass, draw_ui)
        .run();
}

#[allow(deprecated)] // one unavoidable top-level `.show` call; everything nested uses show_inside
fn draw_ui(
    mut contexts: EguiContexts,
    mut scene_texture: ResMut<SceneTexture>,
    // add whatever ui_panels needs, e.g.:
    // db: Res<data::DbHandle>,
) -> Result {
    // Register the Bevy render-target image with egui once, get back a texture id
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
            // ui_panels::class_editor(ui, /* ... */);
        });

        // Remaining central area shows the embedded 3D scene
        scene_viewport::show(ui, scene_texture.as_ref());
    });

    Ok(())
}
