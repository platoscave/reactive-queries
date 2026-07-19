use bevy::asset::{Asset, AssetLoader, LoadContext, io::Reader};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    #[default]
    Loading,
    Loaded,
}

// Define the json data asset type
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct ClassValueAsset(pub serde_json::Value);

// Define the json colors asset type
// What serde reads from the JSON
#[derive(Deserialize)]
struct RawColorEntry {
    name: String,
    color: String,
}

// The actual Bevy asset
#[derive(Asset, TypePath, Debug)]
pub struct ColorMap(pub HashMap<String, Color>);

// Define the asset collection with path attributes
#[derive(AssetCollection, Resource)]
pub struct AssetHandels {
    #[asset(path = "data/classes.json")]
    pub classes_handel: Handle<ClassValueAsset>,

    #[asset(path = "data/colors.json")]
    pub color_map: Handle<ColorMap>,

    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub font: Handle<Font>,

    // Skybox
    #[asset(path = "skyboxes/milkyway/posx.jpg")]
    pub right: Handle<Image>, // +X

    #[asset(path = "skyboxes/milkyway/negx.jpg")]
    pub left: Handle<Image>, // -X

    #[asset(path = "skyboxes/milkyway/posy.jpg")]
    pub top: Handle<Image>, // +Y

    #[asset(path = "skyboxes/milkyway/negy.jpg")]
    pub bottom: Handle<Image>, // -Y

    #[asset(path = "skyboxes/milkyway/posz.jpg")]
    pub front: Handle<Image>, // +Z

    #[asset(path = "skyboxes/milkyway/negz.jpg")]
    pub back: Handle<Image>, // -Z
}

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
pub struct LoadingPlugin;
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app //.register_type::<LoadingPlugin>()
            .init_state::<AppState>()
            .init_asset::<ColorMap>()
            .init_asset_loader::<ColorMapLoader>()
            //.init_asset::<ColorsValueAsset>()
            .add_plugins(JsonAssetPlugin::<ClassValueAsset>::new(&["json"]))
            .add_loading_state(
                LoadingState::new(AppState::Loading)
                    .continue_to_state(AppState::Loaded)
                    .load_collection::<AssetHandels>(),
            );
    }
}

#[derive(Default, TypePath)]
pub struct ColorMapLoader;

impl AssetLoader for ColorMapLoader {
    type Asset = ColorMap;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<ColorMap, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let entries: Vec<RawColorEntry> = serde_json::from_slice(&bytes)?;

        let map = entries
            .into_iter()
            .map(|entry| {
                let hex = entry.color.trim_start_matches('#');
                let r = u8::from_str_radix(&hex[0..2], 16)? as f32 / 255.0;
                let g = u8::from_str_radix(&hex[2..4], 16)? as f32 / 255.0;
                let b = u8::from_str_radix(&hex[4..6], 16)? as f32 / 255.0;
                Ok((entry.name, Color::srgba(r, g, b, 1.0)))
            })
            .collect::<Result<HashMap<_, _>, Self::Error>>()?;

        Ok(ColorMap(map))
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
/*
fn build_skybox(
    mut commands: Commands,
    mut skybox: ResMut<SkyboxFaces>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    camera: Query<Entity, With<Camera3d>>,
) {
    if skybox.spawned {
        return;
    }

    // Wait until every face is loaded
    if skybox.handles.iter().any(|h| !asset_server.is_loaded_with_dependencies(h)) {
        return;
    }

    // Grab pixel data from each face
    let faces: Vec<Image> = skybox.handles.iter()
        .map(|h| images.get(h).expect("face should be loaded").clone())
        .collect();

    let width  = faces[0].width();
    let height = faces[0].height();

    // All faces must be the same size and format
    assert!(faces.iter().all(|f| f.width() == width && f.height() == height),
        "All skybox faces must be the same resolution");

    // Concatenate raw pixel data: face0 data | face1 data | … | face5 data
    let combined: Vec<u8> = faces.iter().flat_map(|f| f.data.iter().copied()).collect();

    // Build a 2D array image with 6 layers
    let mut cubemap = Image::new(
        Extent3d { width, height, depth_or_array_layers: 6 },
        TextureDimension::D2,
        combined,
        TextureFormat::Rgba8UnormSrgb, // match your actual image format
        RenderAssetUsages::RENDER_WORLD,
    );

    // Reinterpret as array, then tell the GPU it's a cube
    cubemap.reinterpret_stacked_2d_as_array(6);
    cubemap.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });

    let cubemap_handle = images.add(cubemap);

    // Attach the skybox to the camera
    for entity in camera.iter() {
        commands.entity(entity).insert(Skybox {
            image: cubemap_handle.clone(),
            brightness: 1000.0,
            rotation: Quat::IDENTITY,
        });
    }

    skybox.spawned = true;
}
*/
