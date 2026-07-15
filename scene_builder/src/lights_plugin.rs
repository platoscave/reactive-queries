use bevy::prelude::*;

pub struct LightsPlugin;
impl Plugin for LightsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 100.0,
            ..default()
        })
        .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 1000.0, // default is 100_000.0
            ..default()
        },
        Transform::from_xyz(-3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
