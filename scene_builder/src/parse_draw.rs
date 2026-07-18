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

/// Marker resource: once inserted, parse_draw_classes has successfully run
/// and won't run again. Prevents re-parsing every frame once it succeeds.
#[derive(Resource)]
struct ClassesParsed;

pub struct ParseDrawPlugin;
impl Plugin for ParseDrawPlugin {
    fn build(&self, app: &mut App) {
        app
            // initialize the registry
            .init_resource::<KeyRegistry>()
            // Retry every frame once Loaded, until the ColorMap asset data
            // is actually available — guards against a real timing gap
            // between "AssetServer reports loaded" and "Assets<ColorMap>
            // actually contains the deserialized value", which showed up
            // on wasm due to its async scheduling (not observed natively,
            // where load latency is near-zero).
            .add_systems(
                Update,
                parse_draw_classes
                    .run_if(in_state(AppState::Loaded))
                    .run_if(not(resource_exists::<ClassesParsed>)),
            )
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
            //.add_systems(OnEnter(AppState::Loaded), parse_draw_classes)
            .add_systems(
                PostUpdate,
                (resolve_pending_associations, update_association_beams)
                    .chain()
                    .after(TransformSystems::Propagate)
                    .run_if(in_state(AppState::Loaded)),
            )
            .add_observer(on_click);

        #[cfg(target_arch = "wasm32")]
        app.add_systems(Update, hide_loading_text)
        ()
    }
}

// app/src/lib.rs — only compiled for wasm, since `web_sys`/DOM APIs don't exist natively
#[cfg(target_arch = "wasm32")]
fn hide_loading_text(classes_parsed: Option<Res<ClassesParsed>>) {
    if classes_parsed.is_some() {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(el) = document.get_element_by_id("loading_text") {
                    el.remove();
                }
            }
        }
    }
}
