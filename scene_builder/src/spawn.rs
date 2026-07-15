#![allow(clippy::too_many_arguments)]
use bevy::prelude::*;
use serde_json::Value;
use std::f32::consts::PI;

use super::components::*;
use crate::parse_draw::*;

pub fn parse_draw_classes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_handels: Res<AssetHandels>,
    color_maps: Res<Assets<ColorMap>>,
    configs: Res<Assets<ClassValueAsset>>,
    mut registry: ResMut<KeyRegistry>,
    assets: Res<AssetHandels>,
) {
    let class_handel = configs
        .get(&asset_handels.classes_handel)
        .expect("classes json not loaded");

    // get the top level array of classes or objects
    let serde_array = class_handel
        .0
        .as_array()
        .expect("serde Value is not an array");

    // for each serde object in the array
    for serde_object in serde_array {
        // get the key for this class/object
        let key = serde_object["key"]
            .as_str()
            .expect("key not found in serde_object");

        // If the object has a classId, its an object, otherwise its a class
        if serde_object.get("classId").is_some() {
            panic!("serde_object is an object (not a class)");
        } else {
            // set optional superclass to None
            let mut superclass_ent_opt: Option<Entity> = None;

            // get the superclass entity if there is one
            let superclass_key_opt = serde_object.get("superClassId");
            if let Some(superclass_key_value) = superclass_key_opt {
                let superclass_key_str = superclass_key_value.as_str().unwrap();

                // lookup the superclass using the key registry
                if let Some(&superclass_ent) = registry.0.get(superclass_key_str) {
                    // set optional superclass to Some
                    superclass_ent_opt = Some(superclass_ent);
                }
            }

            let associations = get_argo_class_titles(serde_object);

            spawn_class(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut registry,
                &assets,
                &color_maps,
                superclass_ent_opt,
                key,
                serde_object["title"].as_str().unwrap_or("unnamed"),
                associations,
            );
        }
    }
}

fn get_argo_class_titles(obj: &Value) -> Vec<(String, String)> {
    let mut result = Vec::new();

    if let Some(properties) = obj.get("properties").and_then(|p| p.as_object()) {
        for (_key, prop) in properties {
            if let Some(argo_query) = prop.get("argoQuery") {
                let class_id = argo_query
                    .get("where")
                    .and_then(|w| w.get("classId"))
                    .and_then(|c| c.as_str());

                //let title = prop.get("title").and_then(|t| t.as_str());
                let title = prop
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unnamed")
                    .to_string();

                // if let (Some(class_id), Some(title)) = (class_id, title) {
                //     result.push((class_id.to_string(), title.to_string()));
                // }

                match class_id {
                    Some(class_id) => {
                        result.push((class_id.to_string(), title));
                    }
                    None => {
                        warn!("no class_id in argoquery");
                    }
                }
            }
        }
    }

    result
}

pub fn spawn_class(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    registry: &mut ResMut<KeyRegistry>,
    assets: &Res<AssetHandels>,
    color_maps: &Res<Assets<ColorMap>>,
    superclass_ent_opt: Option<Entity>,
    key: &str,
    name: &str,
    assosciations: Vec<(String, String)>,
) -> Entity {
    let color_map = color_maps.get(&assets.color_map).unwrap();

    // y offset relative to the parent
    let mut y = -4.0;
    // unless this is the root class
    if superclass_ent_opt.is_none() {
        y = 0.0
    }

    // Spawn Class
    let parent_ent = commands
        .spawn((
            Class, // marker
            //ClassData,
            Subclasses::empty(), // marks this entity as a relationship target
            Collapsed(false),
            Name::new(key.to_owned()),
            Mesh3d(meshes.add(class_mesh().to_bevy(RenderAssetUsages::all()))),
            MeshMaterial3d(materials.add(fadeable_material(*color_map.0.get("class").unwrap()))),
            Transform::from_xyz(0.0, y, 0.0),
            FlyToOnClick,     // Custom tag to identify clickable objects
            HighlightOnHover, // Custom tag to identify hover highlight objects
        )) //.observe(on_class_clicked)// enables picking
        .id();

    // add our key and entity to the registry
    registry.0.insert(key.to_owned(), parent_ent);

    // If we are a subclass, add our id to SubclassOf of the parent entity
    if let Some(superclass_ent) = superclass_ent_opt {
        commands
            .entity(parent_ent)
            .insert(SubclassOf(superclass_ent));
    }

    // Spawn Top Beam
    // If we are a subclass, add a top beam
    if superclass_ent_opt.is_some() {
        let top_beam_ent = commands
            .spawn((
                Name::new("Top Conector"),
                Mesh3d(meshes.add(Cylinder::new(0.05, 2.0).mesh().resolution(50))),
                MeshMaterial3d(
                    materials.add(fadeable_material(*color_map.0.get("classBeam").unwrap())),
                ),
                Transform::from_xyz(0.0, 1.0, 0.0),
            ))
            .id();
        commands.entity(parent_ent).add_child(top_beam_ent);

        // Spawn top sphere
        let top_sphere_ent = commands
            .spawn((
                Name::new("Start Sphere"),
                Mesh3d(meshes.add(Sphere::new(0.05).mesh())),
                MeshMaterial3d(
                    materials.add(fadeable_material(*color_map.0.get("classBeam").unwrap())),
                ),
                Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)),
            ))
            .id();
        commands.entity(parent_ent).add_child(top_sphere_ent);
    }

    // Spawn Horizontal Beam
    let horizontal_beam_ent = commands
        .spawn((
            Name::new("Horizontal Conector"),
            HorizontalBeam,
            Mesh3d(meshes.add(Cylinder::new(0.05, 1.0).mesh().resolution(50))),
            MeshMaterial3d(
                materials.add(fadeable_material(*color_map.0.get("classBeam").unwrap())),
            ),
            Transform::from_xyz(0.0, -2.0, 0.0).with_rotation(Quat::from_rotation_z(PI / 2.0)),
        ))
        .id();
    commands.entity(parent_ent).add_child(horizontal_beam_ent);

    // These entities depend on whether this class has subclasses.
    // We add them later if thts the case
    commands.entity(parent_ent).insert(ClassData {
        beam_entity: horizontal_beam_ent,
        bottom_beam_entity: None,
        collapse_sphere_entity: None,
    });

    // Spawn Name
    let name_ent = commands
        .spawn((
            TextMesh {
                text: name.to_string(),
                font: assets.font.clone(),
                style: TextMeshStyle {
                    depth: 0.0,
                    anchor: TextAnchor::Center,
                    ..default()
                },
            },
            MeshMaterial3d(materials.add(fadeable_material(*color_map.0.get("label").unwrap()))),
            Transform::from_xyz(0.0, 0.0, 0.32).with_scale(Vec3::splat(0.25)),
        ))
        .id();
    commands.entity(parent_ent).add_child(name_ent);

    // If we are a subclass, make our entity a child of our parent
    if let Some(superclass_ent) = superclass_ent_opt {
        commands.entity(superclass_ent).add_child(parent_ent);
    }

    // Record associations to be resolved later, once all classes exist
    // and transforms have propagated (see draw_associations system)
    for (class_id, title) in &assosciations {
        commands.spawn(PendingAssociation {
            from_key: key.to_string(),
            to_class_id: class_id.clone(),
            title: title.clone(),
        });
    }

    parent_ent
}

pub fn fadeable_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        alpha_mode: AlphaMode::Opaque, // starts opaque; animate_alpha switches to Blend only while fading
        ..default()
    }
}
// /////////////////////////////////////////////////////////////////////////////
// Procedural Class mesh, to be replaced by glb import
// /////////////////////////////////////////////////////////////////////////////

use procedural_modelling::extensions::bevy::*;
use procedural_modelling::prelude::*;

fn class_mesh() -> BevyMesh3d {
    let mut mesh = BevyMesh3d::new();

    // Define vertices for a triangle in the XY plane at Z=-0.25
    let vertices = [
        BevyVertexPayload3d::from_pos(Vec3::new(1.0, 1.0 / 6.0, -0.3)),
        BevyVertexPayload3d::from_pos(Vec3::new(0.0, 0.5, -0.3)),
        BevyVertexPayload3d::from_pos(Vec3::new(-1.0, 1.0 / 6.0, -0.3)),
        BevyVertexPayload3d::from_pos(Vec3::new(-1.0, -1.0 / 6.0, -0.3)),
        BevyVertexPayload3d::from_pos(Vec3::new(0.0, -0.5, -0.3)),
        BevyVertexPayload3d::from_pos(Vec3::new(1.0, -1.0 / 6.0, -0.3)),
    ];

    // Insert the polygon by iterating over the vertices
    let mut edge = mesh.insert_polygon(vertices.iter().cloned());

    // Define the extrusion transform (translation and rotation)
    let trans = Transform::from_xyz(0.0, 0.0, 0.1);

    // Extrude along the Z-axis by a depth of 1.0
    edge = mesh.extrude_tri(edge, trans);
    for _ in 0..5 {
        edge = mesh.extrude_tri_face(mesh.edge(edge).face_id(), trans);
    }

    //mesh.to_bevy(RenderAssetUsages::default());
    //mesh.translate(from_xyz(0.0, 0.0, 0.1));

    mesh
}

/*
fn object_mesh() -> BevyMesh3d {
    let mut mesh = BevyMesh3d::new();

    // Define vertices for a triangle in the XY plane at Z=-0.5
    let vertices = [
        BevyVertexPayload3d::from_pos(Vec3::new(1.0, 0.25, -0.3)),
        BevyVertexPayload3d::from_pos(Vec3::new(0.0, 0.75, -0.3)),
        BevyVertexPayload3d::from_pos(Vec3::new(-1.0, 0.25, -0.3)),
        BevyVertexPayload3d::from_pos(Vec3::new(-1.0, -0.25, -0.3)),
        BevyVertexPayload3d::from_pos(Vec3::new(1.0, -0.25, -0.3)),
    ];

    // Insert the polygon by iterating over the vertices
    let mut edge = mesh.insert_polygon(vertices.iter().cloned());

    // Define the extrusion transform (translation and rotation)
    let trans = Transform::from_translation(Vec3::new(0.0, 0.0, 0.1));

    // Extrude along the Z-axis by a depth of 1.0
    edge = mesh.extrude_tri(edge, trans);
    for _ in 0..5 {
        edge = mesh.extrude_tri_face(mesh.edge(edge).face_id(), trans);
    }

    //mesh.transform(&Transform::from_translation(vec3(0.0, 0.0, -0.3)));
    //mesh.to_bevy(RenderAssetUsages::default());

    return mesh;
}
*/
