use bevy::camera::RenderTarget; // <-- moved from bevy::render::camera
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy_egui::egui;

#[derive(Resource)]
pub struct SceneTexture {
    pub image_handle: Handle<Image>,
    pub egui_texture_id: Option<egui::TextureId>,
}

pub fn setup_render_target(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let size = Extent3d {
        width: 800,
        height: 600,
        depth_or_array_layers: 1,
    };

    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    commands.spawn((
        Camera3d::default(),
        RenderTarget::Image(image_handle.clone().into()), // <-- separate component now, not Camera { target: ... }
    ));

    commands.insert_resource(SceneTexture {
        image_handle,
        egui_texture_id: None,
    });
}

pub fn show(ui: &mut egui::Ui, scene_texture: &SceneTexture) {
    if let Some(tex_id) = scene_texture.egui_texture_id {
        let available = ui.available_size();
        ui.image(egui::load::SizedTexture::new(tex_id, available));
    }
}
