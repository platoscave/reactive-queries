use bevy::prelude::*;
use std::collections::HashMap;

// Key registry
// this registry maps our keys to corresponding Entities so that we can more easily look them up
#[derive(Resource, Default)]
pub struct KeyRegistry(pub HashMap<String, Entity>);

// Relationships, these are similar to the builtin parent-child relationship

// SubclassOf goes on the child — points to its parent
#[derive(Component, Clone)]
#[relationship(relationship_target = Subclasses)]
pub struct SubclassOf(pub Entity);

// Subclasses goes on the parent — Bevy auto-manages the collection
#[derive(Component)]
#[relationship_target(relationship = SubclassOf)]
pub struct Subclasses(Vec<Entity>);
// We need to add this because Vec<Entity> is private
impl Subclasses {
    pub(crate) fn empty() -> Self {
        Subclasses(Vec::new())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter()
    }
    #[allow(unused)]
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

// The animate components

// Allows us to annimate the class positions, according to number of subclasses
#[derive(Component)]
pub struct AnimateTranslation {
    pub from: Vec3,
    pub to: Vec3,
    pub elapsed: f32,
    pub duration: f32,
}

// Allows us to annimate the horizontal beam width, according to number of subclasses
#[derive(Component)]
pub struct AnimateScale {
    pub from: Vec3,
    pub to: Vec3,
    pub elapsed: f32,
    pub duration: f32,
}

// Allows us to annimate transparency, fade components in and out
#[derive(Component)]
pub struct AnimateAlpha {
    pub from: f32,
    pub to: f32,
    pub elapsed: f32,
    pub duration: f32,
}

// Tells us whether a component is collapsed or not
#[derive(Component)]
pub struct Collapsed(pub bool);

// Lets the clicked sphere find its owning class entity
#[derive(Component)]
pub struct CollapseOnClick {
    pub class_entity: Entity,
}

// This is our Class component marker (Maybe redundant)
#[derive(Component, Debug)]
pub struct Class;

// Holds on to horizontal beam, bottom beam, collapse sphere
// so that we can spawn and despawn them depending on whether or not the class has subslasses
#[derive(Component, Debug)]
pub struct ClassData {
    pub beam_entity: Entity,                    // horizontal beam, always exists
    pub bottom_beam_entity: Option<Entity>,     // only exists when has children
    pub collapse_sphere_entity: Option<Entity>, // only exists when has children
}

// Marker components. Help us filter during querying

// marker denotes that the component is clickable for camera fly to
#[derive(Component, Debug)]
pub struct FlyToOnClick;

// marker denotes that the component will highlight when hovered over
#[derive(Component, Debug)]
pub struct HighlightOnHover;

// marker denotes the horizontal beam wich grows as the number of children do
#[derive(Component, Debug)]
pub struct HorizontalBeam;
