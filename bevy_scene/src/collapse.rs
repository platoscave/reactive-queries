#![allow(clippy::too_many_arguments)]
use bevy::prelude::*;

use super::components::*;
use crate::parse_draw::*;

// /////////////////////////////////////////////////////////////////////////////
// Collapse subclasses on click
// /////////////////////////////////////////////////////////////////////////////

pub fn on_click(
    mut trigger: On<Pointer<Click>>,
    collapse_handles: Query<&CollapseOnClick>,
    mut collapsed_query: Query<&mut Collapsed>,
) {
    trigger.propagate(false);

    let clicked_entity = trigger.entity;

    let Ok(handle) = collapse_handles.get(clicked_entity) else {
        return; // clicked something other than a collapse sphere
    };

    if let Ok(mut collapsed) = collapsed_query.get_mut(handle.class_entity) {
        collapsed.0 = !collapsed.0;
    }
}

pub fn toggle_collapse_visuals(
    mut commands: Commands,
    changed_collapsed: Query<(Entity, &Collapsed), Changed<Collapsed>>,
    subclasses_query: Query<&Subclasses>,
    children_query: Query<&Children>,
    has_material_query: Query<(), With<MeshMaterial3d<StandardMaterial>>>, // filter-only, for descendant walk
    material_handle_query: Query<&MeshMaterial3d<StandardMaterial>>, // actual handle, for color change
    mut materials: ResMut<Assets<StandardMaterial>>,
    color_maps: Res<Assets<ColorMap>>,
    assets: Res<AssetHandels>,
    class_data_query: Query<&ClassData>,
) {
    let Some(color_map) = color_maps.get(&assets.color_map) else {
        return;
    };

    for (entity, collapsed) in &changed_collapsed {
        let mut descendants = Vec::new();
        collect_fadeable_descendants(
            entity,
            &subclasses_query,
            &children_query,
            &has_material_query, // <-- filter-only query goes here
            &mut descendants,
        );
        let (from_alpha, to_alpha) = if collapsed.0 { (1.0, 0.0) } else { (0.0, 1.0) };

        for descendant in descendants {
            if !collapsed.0 {
                // make visible immediately so the fade-in is seen
                commands.entity(descendant).insert(Visibility::Inherited);
            }
            commands.entity(descendant).insert(AnimateAlpha {
                from: from_alpha,
                to: to_alpha,
                elapsed: 0.0,
                duration: 0.5,
            });
        }

        // Flip the sphere's own color
        if let Ok(class_data) = class_data_query.get(entity)
            && let Some(sphere_ent) = class_data.collapse_sphere_entity
        {
            let color_key = if collapsed.0 { "happy" } else { "unhappy" };
            if let Some(color) = color_map.0.get(color_key)
                && let Ok(material_handle) = material_handle_query.get(sphere_ent)
                && let Some(mut material) = materials.get_mut(&material_handle.0)
            {
                material.base_color = *color;
            }
        }
    }
}

fn collect_fadeable_descendants(
    entity: Entity,
    subclasses_query: &Query<&Subclasses>,
    children_query: &Query<&Children>,
    has_material_query: &Query<(), With<MeshMaterial3d<StandardMaterial>>>,
    out: &mut Vec<Entity>,
) {
    // Only walk into subclasses — never the clicked entity's own direct children
    // (its beam/sphere/label/mesh should stay visible; only the beam scaling
    // to zero handles that visually)
    if let Ok(subclasses) = subclasses_query.get(entity) {
        for &subclass_ent in subclasses.iter() {
            collect_class_and_descendants(
                subclass_ent,
                subclasses_query,
                children_query,
                has_material_query,
                out,
            );
        }
    }
}

// Adds this entity's own visual pieces plus everything in its subtree.
// Used for every subclass under the collapsed node — their own beams/
// spheres/labels DO need to fade, unlike the top-level clicked entity's.
fn collect_class_and_descendants(
    entity: Entity,
    subclasses_query: &Query<&Subclasses>,
    children_query: &Query<&Children>,
    has_material_query: &Query<(), With<MeshMaterial3d<StandardMaterial>>>,
    out: &mut Vec<Entity>,
) {
    if has_material_query.get(entity).is_ok() {
        out.push(entity);
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            if has_material_query.get(child).is_ok() {
                out.push(child);
            }
            collect_class_and_descendants(
                child,
                subclasses_query,
                children_query,
                has_material_query,
                out,
            );
        }
    }

    if let Ok(subclasses) = subclasses_query.get(entity) {
        for &subclass_ent in subclasses.iter() {
            collect_class_and_descendants(
                subclass_ent,
                subclasses_query,
                children_query,
                has_material_query,
                out,
            );
        }
    }
}

pub fn animate_alpha(
    mut commands: Commands,
    time: Res<Time>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(Entity, &mut AnimateAlpha)>,
) {
    for (entity, mut anim) in &mut query {
        anim.elapsed += time.delta_secs();
        let t = (anim.elapsed / anim.duration).clamp(0.0, 1.0);
        let alpha = anim.from + (anim.to - anim.from) * t;

        if let Ok(material_handle) = material_query.get(entity)
            && let Some(mut material) = materials.get_mut(&material_handle.0)
        {
            let mut srgba = material.base_color.to_srgba();
            srgba.alpha = alpha;
            material.base_color = srgba.into();

            // Only blend while actually transparent; opaque otherwise
            // for correct depth sorting/occlusion at rest.
            material.alpha_mode = if alpha < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            };
        }

        if t >= 1.0 {
            commands.entity(entity).remove::<AnimateAlpha>();
            if anim.to <= 0.0 {
                commands.entity(entity).insert(Visibility::Hidden);
            }
        }
    }
}
