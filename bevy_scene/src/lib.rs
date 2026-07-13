use bevy::prelude::*;
use bevy::input::common_conditions::input_toggle_active;
use bevy_egui::EguiPlugin;
use bevy_fontmesh::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod parse_draw;
mod loading_plugin;
mod skybox_camera_plugin;
mod lights_plugin;
mod skybox_plugin;
pub use parse_draw::ParseDrawPlugin;
pub use loading_plugin::LoadingPlugin;
pub use skybox_camera_plugin::SkyboxFlytoCameraPlugin;
pub use lights_plugin::LightsPlugin;

/*
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        // Lights
        .add_plugins(LightsPlugin)
        // loads assets when in Loading state
        .add_plugins(LoadingPlugin)
        // Skybox, Camera, Fly to clicked
        .add_plugins(SkyboxFlytoCameraPlugin)
        // draws model when in Classes state
        .add_plugins(ParseDrawPlugin)
        .run();
}
*/