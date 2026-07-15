use bevy::prelude::*;

use super::components::*;
use crate::parse_draw::*;

#[derive(Component)]
pub struct PendingAssociation {
    pub from_key: String,
    pub to_class_id: String,
    pub title: String,
}

#[derive(Component)]
pub struct ResolvedAssociation {
    pub from_ent: Entity,
    pub to_ent: Entity,
    pub assoc_beam_entity: Entity,
    pub start_beam_entity: Entity,
    pub start_sphere_entity: Entity,
    pub end_beam_entity: Entity,
    pub end_sphere_entity: Entity,
    pub arrow_entity: Entity,
    pub label_entity: Entity,
    pub z_offset: f32,
}

impl ResolvedAssociation {
    pub fn all_visual_entities(&self) -> [Entity; 6] {
        [
            self.assoc_beam_entity,
            self.start_beam_entity,
            self.start_sphere_entity,
            self.end_beam_entity,
            self.end_sphere_entity,
            self.arrow_entity,
        ]
    }
}

#[derive(Component, Debug)]
pub struct AssociationBeam;

// /////////////////////////////////////////////////////////////////////////////
// Draw associations.
// This done after all of the classes have been positioned
// /////////////////////////////////////////////////////////////////////////////

// Step 1: resolve pending associations into persistent, tracked ones.
// Spawns the beam + label entities once, but does NOT despawn the marker —
// it converts it into a ResolvedAssociation for ongoing updates.
pub fn resolve_pending_associations(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    color_maps: Res<Assets<ColorMap>>,
    assets: Res<AssetHandels>,
    query: Query<(Entity, &PendingAssociation)>,
    registry: Res<KeyRegistry>,
) {
    let Some(color_map) = color_maps.get(&assets.color_map) else {
        return;
    };

    for (assoc_entity, assoc) in &query {
        let z_offset = z_offset_from_string(&assoc.title);

        let Some(&from_ent) = registry.0.get(&assoc.from_key) else {
            warn!("Key not found in key registry: {}", assoc.from_key);
            commands.entity(assoc_entity).despawn();
            continue;
        };
        let Some(&to_ent) = registry.0.get(&assoc.to_class_id) else {
            warn!(
                "Association destination not found in key registry: {}",
                assoc.to_class_id
            );
            commands.entity(assoc_entity).despawn();
            continue;
        };

        // Spawn beam with a placeholder transform — update_association_beams
        // will correct it every frame from here on
        let assoc_beam_entity = commands
            .spawn((
                Name::new("Association Beam"),
                AssociationBeam,
                Mesh3d(meshes.add(Cylinder::new(0.05, 1.0).mesh().resolution(50))),
                MeshMaterial3d(
                    materials.add(fadeable_material(pastel_color_from_string(&assoc.title))),
                ),
                Transform::default(),
            ))
            .id();
        commands.entity(from_ent).add_child(assoc_beam_entity);

        // Spawn start beam
        let start_beam_ent = commands
            .spawn((
                Name::new("Start Beam"),
                Mesh3d(meshes.add(Cylinder::new(0.05, z_offset.abs()).mesh().resolution(50))),
                MeshMaterial3d(
                    materials.add(fadeable_material(pastel_color_from_string(&assoc.title))),
                ),
                Transform::from_translation(Vec3::new(0.0, 0.0, z_offset / 2.0))
                    .with_rotation(Quat::from_rotation_x(PI / 2.0)),
            ))
            .id();
        commands.entity(from_ent).add_child(start_beam_ent);

        // Spawn start sphere
        let start_sphere_ent = commands
            .spawn((
                Name::new("Start Sphere"),
                Mesh3d(meshes.add(Sphere::new(0.05).mesh())),
                MeshMaterial3d(
                    materials.add(fadeable_material(pastel_color_from_string(&assoc.title))),
                ),
                Transform::from_translation(Vec3::new(0.0, 0.0, z_offset)),
            ))
            .id();
        commands.entity(from_ent).add_child(start_sphere_ent);

        // Spawn end beam
        let end_beam_ent = commands
            .spawn((
                Name::new("End Beam"),
                Mesh3d(meshes.add(Cylinder::new(0.05, z_offset.abs()).mesh().resolution(50))),
                MeshMaterial3d(
                    materials.add(fadeable_material(pastel_color_from_string(&assoc.title))),
                ),
                Transform::from_translation(Vec3::new(0.0, 0.0, z_offset / 2.0))
                    .with_rotation(Quat::from_rotation_x(PI / 2.0)),
            ))
            .id();
        commands.entity(to_ent).add_child(end_beam_ent);

        // Spawn end sphere
        let end_sphere_ent = commands
            .spawn((
                Name::new("End Sphere"),
                Mesh3d(meshes.add(Sphere::new(0.05).mesh())),
                MeshMaterial3d(
                    materials.add(fadeable_material(pastel_color_from_string(&assoc.title))),
                ),
                Transform::from_translation(Vec3::new(0.0, 0.0, z_offset)),
            ))
            .id();
        commands.entity(to_ent).add_child(end_sphere_ent);

        // Spawn arrow
        let arrow_ent = commands
            .spawn((
                Name::new("Arrow"),
                Mesh3d(meshes.add(Cone::new(0.25, 0.6).mesh())),
                MeshMaterial3d(
                    materials.add(fadeable_material(pastel_color_from_string(&assoc.title))),
                ),
                Transform::from_translation(Vec3::new(0.0, 0.0, -0.5))
                    .with_rotation(Quat::from_rotation_x(PI / 2.0)),
            ))
            .id();
        commands.entity(to_ent).add_child(arrow_ent);

        let name_ent = commands
            .spawn((
                TextMesh {
                    text: assoc.title.clone(),
                    font: assets.font.clone(),
                    style: TextMeshStyle {
                        depth: 0.0,
                        anchor: TextAnchor::Center,
                        ..default()
                    },
                },
                MeshMaterial3d(
                    materials.add(fadeable_material(*color_map.0.get("label").unwrap())),
                ),
                Transform::default().with_scale(Vec3::splat(0.25)),
            ))
            .id();
        commands.entity(from_ent).add_child(name_ent);

        // Replace PendingAssociation with a persistent ResolvedAssociation
        // so update_association_beams keeps this beam correct every frame.
        // We stash both entity ids so we can find/update the beam mesh directly.
        let z_offset = z_offset_from_string(&assoc.title);

        commands
            .entity(assoc_entity)
            .remove::<PendingAssociation>()
            .insert(ResolvedAssociation {
                from_ent,
                to_ent,
                assoc_beam_entity,
                start_beam_entity: start_beam_ent,
                start_sphere_entity: start_sphere_ent,
                end_beam_entity: end_beam_ent,
                end_sphere_entity: end_sphere_ent,
                arrow_entity: arrow_ent,
                label_entity: name_ent,
                z_offset,
            });
    }
}

// Step 2: every frame, recompute each resolved beam's length/rotation/position
// from current GlobalTransforms. Cheap for typical diagram sizes, and immune
// to any layout timing issues since it just keeps correcting itself.

pub fn update_association_beams(
    resolved_query: Query<&ResolvedAssociation>,
    global_transforms: Query<&GlobalTransform>,
    mut beam_transforms: Query<&mut Transform, (With<AssociationBeam>, Without<TextMesh>)>,
    mut label_transforms: Query<&mut Transform, With<TextMesh>>,
) {
    for resolved in &resolved_query {
        let (Ok(from_global), Ok(to_global)) = (
            global_transforms.get(resolved.from_ent),
            global_transforms.get(resolved.to_ent),
        ) else {
            continue;
        };

        let local_transform = to_global.reparented_to(from_global);
        let target = local_transform.translation;

        let length = target.length();
        if length < f32::EPSILON {
            continue;
        }

        let direction = target / length;
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

        let mut midpoint = target * 0.5;
        midpoint.z += resolved.z_offset;

        if let Ok(mut beam_transform) = beam_transforms.get_mut(resolved.assoc_beam_entity) {
            beam_transform.translation = midpoint;
            beam_transform.rotation = rotation;
            beam_transform.scale = Vec3::new(1.0, length, 1.0);
        }

        if let Ok(mut label_transform) = label_transforms.get_mut(resolved.label_entity) {
            let mut label_pos = midpoint;
            label_pos.z += 0.1;
            label_transform.translation = label_pos;
        }
    }
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn pastel_color_from_string(s: &str) -> Color {
    // Hash the string into a deterministic u64
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish();

    // Map the hash to a hue in [0, 360)
    let hue = (hash % 360) as f32;

    // Fixed saturation/lightness tuned for a pastel look
    let saturation = 0.55; // 0.0 = gray, 1.0 = fully saturated
    let lightness = 0.50; // higher = lighter/pastel-er

    Color::hsl(hue, saturation, lightness)
}

fn z_offset_from_string(s: &str) -> f32 {
    // Same FNV-1a hash as pastel_color_from_string, for consistency
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    // Map hash to a float in [-4.0, -1.0)
    let normalized = (hash % 1000) as f32 / 1000.0; // [0.0, 1.0)
    -1.0 - normalized * 3.0 // [-1.0, -4.0)
}
