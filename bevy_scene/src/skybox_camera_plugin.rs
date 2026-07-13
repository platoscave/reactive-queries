#![allow(clippy::too_many_arguments)]

use crate::loading_plugin::*;
use crate::parse_draw::FlyToOnClick;
use crate::parse_draw::HighlightOnHover;
use crate::parse_draw::ResolvedAssociation;
use crate::skybox_plugin::*;
use bevy::prelude::Pointer;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
//use bevy_skybox::{SkyboxCamera, SkyboxPlugin};

//pub use skybox_plugin::*;

#[derive(Component)]
pub struct MainCamera;

// The animate component
#[derive(Component, Debug, Default)]
pub struct AnimatePanOrbit {
    start_focus: Vec3,
    start_radius: f32,
    start_pitch: f32,
    start_yaw: f32,
    target_focus: Vec3,
    target_radius: f32,
    target_pitch: f32,
    target_yaw: f32,
    pub elapsed: f32,
    pub duration: f32,
}

// OnEnter(Loaded)
//     └─ setup_scene        ← spawns Camera3d + PanOrbitCamera, inserts SkyboxImages
//     └─ build_skybox       ← (SkyboxSystem set) stitches cubemap, attaches Skybox to camera
pub struct SkyboxFlytoCameraPlugin;
impl Plugin for SkyboxFlytoCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            //.add_plugins(SkyboxPlugin::from_image_file("skyboxes/sky1.png"))
            .add_systems(PostUpdate, animate_pan_orbit)
            .add_plugins(PanOrbitCameraPlugin)
            .add_plugins(SixImageSkyboxPlugin)
            .add_observer(on_click)
            .add_observer(on_hover_start)
            .add_observer(on_hover_end)
            .add_systems(
                OnEnter(AppState::Loaded),
                setup.before(SkyboxSystem), // <-- ordering guarantee
            );
    }
}

fn setup(mut commands: Commands, assets: Res<AssetHandels>) {
    // Spawn the camera — PanOrbitCamera works with Skybox since
    // Skybox is just a component on the Camera3d entity
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        //PanOrbitCamera::default(),
        //Transform::from_xyz(0.0, -6.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            // Enables camera controls
            focus: Vec3::new(0.0, -6.0, 0.0), // the point to orbit around
            radius: Some(20.0),               // distance from focus
            pitch: Some(0.0),                 // tilt up/down in radians
            yaw: Some(0.0),                   // rotation left/right in radians
            ..default()
        },
    ));

    // Provide the images the plugin needs
    commands.insert_resource(
        SkyboxImages::new(
            assets.right.clone(),
            assets.left.clone(),
            assets.top.clone(),
            assets.bottom.clone(),
            assets.front.clone(),
            assets.back.clone(),
        )
        .with_brightness(250.0),
    );
}

// /////////////////////////////////////////////////////////////////////////////
// Fly to on click
// /////////////////////////////////////////////////////////////////////////////
fn on_click(
    mut trigger: On<Pointer<Click>>,
    mut commands: Commands,
    flyto_query: Query<(), With<FlyToOnClick>>,
    clicked_transforms: Query<&GlobalTransform>,
    mut camera_query: Query<(Entity, &mut PanOrbitCamera), With<MainCamera>>,
    resolved_query: Query<&ResolvedAssociation>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    previously_selected: Query<Entity, With<Selected>>,
    selected_query: Query<(), With<Selected>>,
    original_query: Query<&OriginalEmissive>,
) {
    trigger.propagate(false);

    let clicked_entity = trigger.entity;

    // Only respond if this entity opted in to fly to
    if flyto_query.get(clicked_entity).is_err() {
        return;
    }

    let Ok(clicked_global) = clicked_transforms.get(clicked_entity) else {
        return;
    };
    let clicked_translation = clicked_global.translation();

    let Ok((camera_entity, mut pan_orbit)) = camera_query.single_mut() else {
        println!("camera not found");
        return;
    };

    // Deselect whatever was previously selected and restore its emissive
    for prev in &previously_selected {
        commands.entity(prev).remove::<Selected>();
        restore_entity(
            &mut commands,
            prev,
            &material_query,
            &mut materials,
            &original_query,
            &selected_query,
        );
    }

    // Select the newly clicked entity
    commands.entity(clicked_entity).insert(Selected);
    highlight_entity(
        &mut commands,
        clicked_entity,
        false, // not hovered, just selected
        &material_query,
        &mut materials,
        &original_query,
        &selected_query,
    );

    // Select every visual piece of any association connected to this entity
    for resolved in &resolved_query {
        if resolved.from_ent == clicked_entity || resolved.to_ent == clicked_entity {
            for beam_entity in resolved.all_visual_entities() {
                commands.entity(beam_entity).insert(Selected);
                highlight_entity(
                    &mut commands,
                    beam_entity,
                    false,
                    &material_query,
                    &mut materials,
                    &original_query,
                    &selected_query,
                );
            }
        }
    }

    //println!("clicked_translation: {:?}", clicked_translation);

    pan_orbit.enabled = false;
    commands.entity(camera_entity).insert(AnimatePanOrbit {
        start_focus: pan_orbit.focus,
        start_radius: pan_orbit.radius.unwrap_or(20.0),
        start_pitch: pan_orbit.pitch.unwrap_or(0.0),
        start_yaw: pan_orbit.yaw.unwrap_or(0.0),
        target_focus: clicked_translation,
        target_radius: 10.0,
        target_pitch: 0.0,
        target_yaw: 0.0,
        elapsed: 0.0,
        duration: 1.5,
    });
}

fn animate_pan_orbit(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PanOrbitCamera, &mut AnimatePanOrbit)>,
) {
    for (entity, mut pan_orbit, mut anim) in &mut query {
        anim.elapsed += time.delta_secs();
        let t = (anim.elapsed / anim.duration).clamp(0.0, 1.0);

        let eased = 1.0 - (1.0 - t) * (1.0 - t);

        let current_focus = anim.start_focus.lerp(anim.target_focus, eased);
        let current_radius = anim.start_radius + (anim.target_radius - anim.start_radius) * eased;
        let current_pitch = anim.start_pitch + (anim.target_pitch - anim.start_pitch) * eased;
        let current_yaw = anim.start_yaw + (anim.target_yaw - anim.start_yaw) * eased;

        pan_orbit.target_focus = current_focus;
        pan_orbit.target_radius = current_radius;
        pan_orbit.target_pitch = current_pitch;
        pan_orbit.target_yaw = current_yaw;

        // Let the plugin compute the Transform itself, every frame,
        // using its own internal formula — consistently, start to finish.
        pan_orbit.force_update = true;

        // println!("current focus: {:?}", current_focus);
        if t >= 1.0 {
            //println!("final focus: {:?}", current_focus);

            commands.entity(entity).remove::<AnimatePanOrbit>();
            pan_orbit.enabled = true; // hand control back to the user
        }
    }
}

// /////////////////////////////////////////////////////////////////////////////
// Highlight on hover over
// /////////////////////////////////////////////////////////////////////////////
#[derive(Component)]
pub struct OriginalEmissive(pub LinearRgba);

#[derive(Component)]
pub struct Selected;

const SELECTED_EMISSIVE_BOOST: f32 = 1.0;
const HOVER_EMISSIVE_BOOST: f32 = 0.5; // used when hovered, whether or not also selected
const HOVER_ON_SELECTED_EXTRA: f32 = 1.0; // extra multiplier when hovering the already-selected entity

fn tint_for(material: &StandardMaterial) -> LinearRgba {
    if material.base_color != Color::BLACK {
        LinearRgba::from(material.base_color)
    } else {
        LinearRgba::WHITE
    }
}

// Recompute and apply the correct emissive for an entity based on its current state
fn apply_emissive_state(
    entity: Entity,
    is_hovered: bool,
    material_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    materials: &mut Assets<StandardMaterial>,
    selected_query: &Query<(), With<Selected>>,
) {
    let Ok(material_handle) = material_query.get(entity) else {
        return;
    };
    let Some(mut material) = materials.get_mut(&material_handle.0) else {
        return;
    };

    let tint = tint_for(&material);
    let is_selected = selected_query.get(entity).is_ok();

    material.emissive = match (is_selected, is_hovered) {
        (true, true) => tint * SELECTED_EMISSIVE_BOOST * HOVER_ON_SELECTED_EXTRA,
        (true, false) => tint * SELECTED_EMISSIVE_BOOST,
        (false, true) => tint * HOVER_EMISSIVE_BOOST,
        (false, false) => LinearRgba::BLACK, // handled by restore-to-original elsewhere
    };
}

fn highlight_entity(
    commands: &mut Commands,
    entity: Entity,
    is_hovered: bool,
    material_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    materials: &mut Assets<StandardMaterial>,
    original_query: &Query<&OriginalEmissive>,
    selected_query: &Query<(), With<Selected>>,
) {
    if original_query.get(entity).is_err()
        && let Ok(material_handle) = material_query.get(entity)
        && let Some(material) = materials.get(&material_handle.0)
    {
        commands
            .entity(entity)
            .insert(OriginalEmissive(material.emissive));
    }

    apply_emissive_state(
        entity,
        is_hovered,
        material_query,
        materials,
        selected_query,
    );
}

fn on_hover_start(
    mut trigger: On<Pointer<Over>>,
    mut commands: Commands,
    highlightable_query: Query<(), With<HighlightOnHover>>,
    resolved_query: Query<&ResolvedAssociation>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    original_query: Query<&OriginalEmissive>,
    selected_query: Query<(), With<Selected>>,
) {
    trigger.propagate(false);

    let entity = trigger.entity;

    // Only respond if this entity opted in to hover highlighting
    if highlightable_query.get(entity).is_err() {
        return;
    }

    // Highlight the hovered entity itself
    highlight_entity(
        &mut commands,
        entity,
        true,
        &material_query,
        &mut materials,
        &original_query,
        &selected_query,
    );

    // Highlight every visual piece of any association connected to this entity
    for resolved in &resolved_query {
        if resolved.from_ent == entity || resolved.to_ent == entity {
            for beam_entity in resolved.all_visual_entities() {
                highlight_entity(
                    &mut commands,
                    beam_entity,
                    true,
                    &material_query,
                    &mut materials,
                    &original_query,
                    &selected_query,
                );
            }
        }
    }
}

fn restore_entity(
    commands: &mut Commands,
    entity: Entity,
    material_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    materials: &mut Assets<StandardMaterial>,
    original_query: &Query<&OriginalEmissive>,
    selected_query: &Query<(), With<Selected>>,
) {
    let is_selected = selected_query.get(entity).is_ok();

    if is_selected {
        // Drop back to "selected" glow level, not all the way to original
        apply_emissive_state(entity, false, material_query, materials, selected_query);
    } else {
        // Fully restore original emissive and drop the tracking component
        if let Ok(material_handle) = material_query.get(entity)
            && let Some(mut material) = materials.get_mut(&material_handle.0)
            && let Ok(original) = original_query.get(entity)
        {
            material.emissive = original.0;
        }
        commands.entity(entity).remove::<OriginalEmissive>();
    }
}

fn on_hover_end(
    mut trigger: On<Pointer<Out>>,
    mut commands: Commands,
    highlightable_query: Query<(), With<HighlightOnHover>>,
    resolved_query: Query<&ResolvedAssociation>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    original_query: Query<&OriginalEmissive>,
    selected_query: Query<(), With<Selected>>,
) {
    trigger.propagate(false);

    let entity = trigger.entity;

    // Only respond if this entity opted in to hover highlighting
    if highlightable_query.get(entity).is_err() {
        return;
    }

    // Restore the hovered entity itself
    restore_entity(
        &mut commands,
        entity,
        &material_query,
        &mut materials,
        &original_query,
        &selected_query,
    );

    // Restore every visual piece of any association connected to this entity
    for resolved in &resolved_query {
        if resolved.from_ent == entity || resolved.to_ent == entity {
            for beam_entity in resolved.all_visual_entities() {
                restore_entity(
                    &mut commands,
                    beam_entity,
                    &material_query,
                    &mut materials,
                    &original_query,
                    &selected_query,
                );
            }
        }
    }
}
