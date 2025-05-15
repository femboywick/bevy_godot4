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
    pub use super::erased_gd::{ErasedGd, ErasedGdResource};
    pub use super::events::SignalEventEmpty;
    pub use super::scene::GodotScene;
    pub use super::scene_tree::SceneTreeRef;
    pub use super::utils::{
        AsPhysicsSystem, AsVisualSystem, GodotPhysicsFrame, GodotVisualFrame, SystemDeltaTimer,
    };
    pub use bevy_godot4_proc_macros::bevy_app;
}
pub use app::{APP_BUILDER_FN, BevyApp};
