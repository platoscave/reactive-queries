// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Bevy game template stuff
use bevy::DefaultPlugins;
use bevy::asset::AssetMetaCheck;
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WINIT_WINDOWS;
//use bevy_game::GamePlugin; // ToDo: Replace bevy_game with your new crate name.
use std::io::Cursor;
use winit::window::Icon;

// Our stuff
use bevy::input::common_conditions::input_toggle_active;
//use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_fontmesh::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod lights_plugin;
mod loading_plugin;
mod parse_draw;
mod skybox_camera_plugin;
mod skybox_plugin;
pub use lights_plugin::LightsPlugin;
pub use loading_plugin::LoadingPlugin;
pub use parse_draw::ParseDrawPlugin;
pub use skybox_camera_plugin::SkyboxFlytoCameraPlugin;

fn main() {
    App::new()
        // Bevy game template stuff
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Observable Query Results".to_string(), // ToDo
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()),
                        fit_canvas_to_parent: true,
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                // Our stuff
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        // Our stuff
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
        .add_systems(Startup, set_window_icon)
        .run();
}

// Sets the icon on windows and X11
fn set_window_icon(
    primary_window: Single<Entity, With<PrimaryWindow>>,
    _non_send_marker: NonSendMarker,
) -> Result {
    WINIT_WINDOWS.with_borrow(|windows| {
        let Some(primary) = windows.get_window(*primary_window) else {
            return Err(BevyError::from("No primary window!"));
        };
        let icon_buf = Cursor::new(include_bytes!(
            "../build/macos/AppIcon.iconset/icon_256x256.png"
        ));
        if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
            let image = image.into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            let icon = Icon::from_rgba(rgba, width, height).unwrap();
            primary.set_window_icon(Some(icon));
        };

        Ok(())
    })
}
