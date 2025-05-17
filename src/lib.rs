#![feature(fn_traits)]
mod app;
#[cfg(feature = "assets")]
mod assets;
mod erased_gd;
mod events;
mod scene;
mod scene_tree;
mod utils;

pub mod prelude {
    pub use super::erased_gd::{ErasedGd, ErasedGdResource, TypedErasedGd};
    pub use super::events::SignalEventEmpty;
    pub use super::scene::GodotScene;
    pub use super::scene_tree::SceneTreeRef;
    pub use super::utils::{
        AsPhysicsSystem, AsVisualSystem, GodotPhysicsFrame, GodotVisualFrame, SystemDeltaTimer,
    };
    pub use crate::bevy_spawn;
    pub use bevy_godot4_proc_macros::{bevy_app, signal_event, signal_event_instanced};
}
pub use app::{APP_BUILDER_FN, BevyApp};
