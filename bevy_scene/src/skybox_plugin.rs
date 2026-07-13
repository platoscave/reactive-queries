use bevy::asset::RenderAssetUsages;
use bevy::core_pipeline::Skybox;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};

use crate::loading_plugin::AppState;

/// Insert this resource before the skybox system runs.
#[derive(Resource)]
pub struct SkyboxImages {
    pub right: Handle<Image>,  // +X
    pub left: Handle<Image>,   // -X
    pub top: Handle<Image>,    // +Y
    pub bottom: Handle<Image>, // -Y
    pub front: Handle<Image>,  // +Z
    pub back: Handle<Image>,   // -Z
    pub brightness: f32,
}

impl SkyboxImages {
    pub fn new(
        right: Handle<Image>,
        left: Handle<Image>,
        top: Handle<Image>,
        bottom: Handle<Image>,
        front: Handle<Image>,
        back: Handle<Image>,
    ) -> Self {
        Self {
            right,
            left,
            top,
            bottom,
            front,
            back,
            brightness: 1000.0,
        }
    }

    pub fn with_brightness(mut self, brightness: f32) -> Self {
        self.brightness = brightness;
        self
    }
}

/// System set you can use for ordering: `my_setup.before(SkyboxSystem)`
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkyboxSystem;

pub struct SixImageSkyboxPlugin;

impl Plugin for SixImageSkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loaded), build_skybox.in_set(SkyboxSystem));
    }
}

fn build_skybox(
    mut commands: Commands,
    skybox_images: Res<SkyboxImages>,
    mut images: ResMut<Assets<Image>>,
    camera: Query<Entity, With<Camera3d>>,
) {
    let handles = [
        &skybox_images.right,
        &skybox_images.left,
        &skybox_images.top,
        &skybox_images.bottom,
        &skybox_images.front,
        &skybox_images.back,
    ];

    let face_images: Vec<Image> = handles
        .iter()
        .map(|h| images.get(*h).expect("skybox face not loaded").clone())
        .collect();

    let width = face_images[0].width();
    let height = face_images[0].height();

    let combined: Vec<u8> = face_images
        .iter()
        .flat_map(|f| f.data.as_ref().expect("image has no data").iter().copied())
        .collect();

    let mut cubemap = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        combined,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    let _ = cubemap.reinterpret_stacked_2d_as_array(6);
    cubemap.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });

    let cubemap_handle = images.add(cubemap);
    let brightness = skybox_images.brightness;

    for entity in camera.iter() {
        commands.entity(entity).insert(Skybox {
            image: Some(cubemap_handle.clone()),
            brightness,
            rotation: Quat::IDENTITY,
        });
    }
}
