use crate::*; // for AppState, ColorMap, AssetHandels, ClassValueAsset, etc.
use bevy::asset::RenderAssetUsages;
pub use loading_plugin::*;
use std::f32::consts::PI;

// Our modules, adjacent sources
mod associations;
mod collapse;
mod components;
mod layout;
mod spawn;
// Re-export everything
pub use associations::*; // so crate::parse_draw::ResolvedAssociation still works
use collapse::*;
pub use components::*;
use layout::*;
use spawn::*;

pub struct ParseDrawPlugin;
impl Plugin for ParseDrawPlugin {
    fn build(&self, app: &mut App) {
        app
            // initialize the registry
            .init_resource::<KeyRegistry>()
            .add_systems(
                Update,
                (
                    sync_leaf_visuals, // run before update_subclass_positions
                    update_subclass_positions,
                    animate_translation,
                    animate_scale,
                    animate_alpha,
                    toggle_collapse_visuals,
                )
                    .chain()
                    .run_if(in_state(AppState::Loaded)),
            )
            // parse the classes.json Value and draw the classes
            .add_systems(OnEnter(AppState::Loaded), parse_draw_classes)
            .add_systems(
                PostUpdate,
                (resolve_pending_associations, update_association_beams)
                    .chain()
                    .after(TransformSystems::Propagate)
                    .run_if(in_state(AppState::Loaded)),
            )
            .add_observer(on_click);
    }
}
