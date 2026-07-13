#![allow(clippy::too_many_arguments)]
use bevy::prelude::*;
use std::collections::HashSet;

use super::components::*;
use crate::parse_draw::*;

// /////////////////////////////////////////////////////////////////////////////
// Whenever the Subclasses relationship changes (added or removed),
// or a Collapse Sphere is clicked, recalculate the subclass positions.
// Teporarily add annimation components to smoothly translate classes and
// scale the horizontal beam.
// /////////////////////////////////////////////////////////////////////////////
pub fn update_subclass_positions(
    mut commands: Commands,
    changed_subclasses: Query<Entity, Changed<Subclasses>>,
    changed_collapsed: Query<Entity, Changed<Collapsed>>, // <-- new
    node_query: Query<(&Transform, &ClassData, &Subclasses)>,
    all_subclasses: Query<&Subclasses>,
    collapsed_query: Query<&Collapsed>, // <-- new
    subclass_of: Query<&SubclassOf>,
    transforms: Query<(&Transform, &Name)>,
    beams: Query<&Transform, With<HorizontalBeam>>,
    mut visibility_query: Query<&mut Visibility>,
) {
    let spacing = 4.0;
    let duration = 1.5;

    let roots: HashSet<Entity> = changed_subclasses
        .iter()
        .chain(changed_collapsed.iter())
        .map(|e| find_root(e, &subclass_of))
        .collect();

    for root in roots {
        layout_subtree(
            &mut commands,
            root,
            0.0,
            &all_subclasses,
            &node_query,
            &collapsed_query,
            &transforms,
            &beams,
            &mut visibility_query,
            spacing,
            duration,
        );
    }
}

fn find_root(entity: Entity, subclass_of: &Query<&SubclassOf>) -> Entity {
    let mut current = entity;
    while let Ok(SubclassOf(parent)) = subclass_of.get(current) {
        current = *parent
    }
    current
}

// /////////////////////////////////////////////////////////////////////////////
// Recusivly layout subclasses
// /////////////////////////////////////////////////////////////////////////////

fn layout_subtree(
    commands: &mut Commands,
    entity: Entity,
    center_x: f32,
    subclasses_query: &Query<&Subclasses>,
    node_query: &Query<(&Transform, &ClassData, &Subclasses)>,
    collapsed_query: &Query<&Collapsed>,
    transforms: &Query<(&Transform, &Name)>,
    beams: &Query<&Transform, With<HorizontalBeam>>,
    visibility_query: &mut Query<&mut Visibility>,
    spacing: f32,
    duration: f32,
) -> f32 {
    let Ok((_, class_data, subclasses)) = node_query.get(entity) else {
        return 0.0;
    };

    let is_collapsed = collapsed_query.get(entity).is_ok_and(|c| c.0);
    let has_children = !subclasses.is_empty() && !is_collapsed;

    // Only the horizontal beam needs visibility control — it's collapse-aware
    // and shrinks/hides itself via AnimateScale. Bottom beam and sphere are
    // spawned/despawned by sync_leaf_visuals, so no visibility toggling needed.
    if let Ok(mut vis) = visibility_query.get_mut(class_data.beam_entity) {
        *vis = if has_children {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    if !has_children {
        return 0.0;
    }

    let widths: Vec<f32> = subclasses
        .iter()
        .map(|&child| subtree_width(child, subclasses_query, collapsed_query, spacing).max(spacing))
        .collect();

    let total_width: f32 = widths.iter().sum();
    let mut cursor = center_x - total_width / 2.0;

    let mut first_child_x: Option<f32> = None;
    let mut last_child_x = 0.0;

    for (&child, &w) in subclasses.iter().zip(widths.iter()) {
        let child_center = cursor + w / 2.0;
        first_child_x.get_or_insert(child_center);
        last_child_x = child_center;

        if let Ok((child_transform, _)) = transforms.get(child) {
            commands.entity(child).insert(AnimateTranslation {
                from: child_transform.translation,
                to: Vec3::new(
                    child_center,
                    child_transform.translation.y,
                    child_transform.translation.z,
                ),
                elapsed: 0.0,
                duration,
            });
        }

        // Recurse — child's children are in child-local space, centered at 0
        layout_subtree(
            commands,
            child,
            0.0,
            subclasses_query,
            node_query,
            collapsed_query,
            transforms,
            beams,
            visibility_query,
            spacing,
            duration,
        );

        cursor += w;
    }

    let span = last_child_x - first_child_x.unwrap_or(center_x);
    let midpoint = (first_child_x.unwrap_or(center_x) + last_child_x) / 2.0;

    // Scale this node's horizontal beam to match its children's span
    if let Ok(beam_transform) = beams.get(class_data.beam_entity) {
        commands
            .entity(class_data.beam_entity)
            .insert(AnimateScale {
                from: beam_transform.scale,
                to: Vec3::new(1.0, span, 1.0),
                elapsed: 0.0,
                duration,
            })
            .insert(AnimateTranslation {
                from: beam_transform.translation,
                to: Vec3::new(
                    midpoint,
                    beam_transform.translation.y,
                    beam_transform.translation.z,
                ),
                elapsed: 0.0,
                duration,
            });
    }

    span
}

fn subtree_width(
    entity: Entity,
    subclasses_query: &Query<&Subclasses>,
    collapsed_query: &Query<&Collapsed>, // <-- new
    spacing: f32,
) -> f32 {
    let Ok(subclasses) = subclasses_query.get(entity) else {
        return 0.0;
    };
    let is_collapsed = collapsed_query.get(entity).is_ok_and(|c| c.0);
    if subclasses.is_empty() || is_collapsed {
        return 0.0;
    }
    subclasses
        .iter()
        .map(|&child| subtree_width(child, subclasses_query, collapsed_query, spacing).max(spacing))
        .sum()
}

pub fn sync_leaf_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    color_maps: Res<Assets<ColorMap>>,
    assets: Res<AssetHandels>,
    changed_subclasses: Query<Entity, Changed<Subclasses>>,
    all_subclasses: Query<&Subclasses>,
    mut class_data_query: Query<&mut ClassData>,
) {
    let Some(color_map) = color_maps.get(&assets.color_map) else {
        return;
    };

    for entity in &changed_subclasses {
        let Ok(subclasses) = all_subclasses.get(entity) else {
            continue;
        };
        let has_children = !subclasses.is_empty();

        let Ok(mut class_data) = class_data_query.get_mut(entity) else {
            continue;
        };

        match (has_children, class_data.bottom_beam_entity) {
            (true, None) => {
                // Gained children — spawn the beam + sphere once
                let bottom_beam_ent = commands
                    .spawn((
                        Name::new("Bottom Conector"),
                        Mesh3d(meshes.add(Cylinder::new(0.05, 2.0).mesh().resolution(50))),
                        MeshMaterial3d(
                            materials
                                .add(fadeable_material(*color_map.0.get("classBeam").unwrap())),
                        ),
                        Transform::from_xyz(0.0, -1.0, 0.0),
                    ))
                    .id();
                commands.entity(entity).add_child(bottom_beam_ent);

                let collapse_sphere_ent = commands
                    .spawn((
                        Name::new("Collapse Sphere"),
                        Mesh3d(meshes.add(Sphere::new(0.1).mesh())),
                        MeshMaterial3d(
                            materials.add(fadeable_material(*color_map.0.get("unhappy").unwrap())),
                        ),
                        Transform::from_translation(Vec3::new(0.0, -2.0, 0.0)),
                        HighlightOnHover,
                        CollapseOnClick {
                            class_entity: entity,
                        },
                    ))
                    .id();
                commands.entity(entity).add_child(collapse_sphere_ent);

                class_data.bottom_beam_entity = Some(bottom_beam_ent);
                class_data.collapse_sphere_entity = Some(collapse_sphere_ent);
            }
            (false, Some(bottom_beam_ent)) => {
                // Lost all children (shouldn't normally happen, but handle it) — clean up
                commands.entity(bottom_beam_ent).despawn();
                if let Some(sphere_ent) = class_data.collapse_sphere_entity {
                    commands.entity(sphere_ent).despawn();
                }
                class_data.bottom_beam_entity = None;
                class_data.collapse_sphere_entity = None;
            }
            _ => {} // already in the correct state, nothing to do
        }
    }
}

// Animate the scale of the horizontal beam according to subclasses width

pub fn animate_scale(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut AnimateScale)>,
) {
    for (entity, mut transform, mut anim) in &mut query {
        anim.elapsed += time.delta_secs();

        let t = (anim.elapsed / anim.duration).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - t) * (1.5 - t);

        transform.scale = anim.from.lerp(anim.to, eased);

        if t >= 1.0 {
            commands.entity(entity).remove::<AnimateScale>();
        }
    }
}

// Animate the positions of the classes relative to the width of the subclasses

pub fn animate_translation(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut AnimateTranslation)>,
) {
    for (entity, mut transform, mut anim) in &mut query {
        anim.elapsed += time.delta_secs();

        let t = (anim.elapsed / anim.duration).clamp(0.0, 1.0);

        // Quadratic out: decelerates into the target
        let eased = 1.0 - (1.0 - t) * (1.5 - t);

        transform.translation = anim.from.lerp(anim.to, eased);

        // Remove component when done
        if t >= 1.0 {
            commands.entity(entity).remove::<AnimateTranslation>();
        }
    }
}
