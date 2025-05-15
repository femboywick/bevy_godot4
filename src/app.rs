use bevy::app::App;
use godot::{
    classes::{Engine, INode, Node, SceneTree},
    obj::{Base, Gd},
    prelude::{GodotClass, godot_api},
};

use crate::prelude::*;
use std::{
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    sync::Mutex,
};

// #[derive(GodotClass)]
// #[class(tool, init, base=EditorPlugin)]
// struct BevyAppEditorPlugin {
//     base: Base<EditorPlugin>,
// }

// #[godot_api]
// impl IEditorPlugin for BevyAppEditorPlugin {
//     fn enter_tree(&mut self) {
//         let singleton = Engine::singleton()
//             .get_singleton("BevyAppSingleton")
//             .unwrap()
//             .cast::<BevyAppSingleton>();

//         self.base_mut()
//             .signals()
//             .scene_changed()
//             .connect_obj(&singleton, BevyAppSingleton::on_scene_change);
//     }
// }

// #[derive(GodotClass)]
// #[class(no_init, base=Object)]
// pub struct BevyAppSingleton {
//     pub inner: Gd<BevyApp>,
// }

// impl BevyAppSingleton {
//     pub fn new(object: Gd<BevyApp>) -> Gd<Self> {
//         Gd::from_object(Self { inner: object })
//     }

//     pub fn get(&self) -> Gd<BevyApp> {
//         self.inner.clone()
//     }

//     fn on_scene_change(&mut self, mut node: Gd<Node>) {
//         let mut inner = self.get();
//         if inner.get_parent().is_none() {
//             node.add_child(&inner);
//         } else {
//             inner.reparent(&node);
//         }
//         godot_print!("{}", node.get_tree_string_pretty());
//         inner.request_ready();
//     }
// }

lazy_static::lazy_static! {
    #[doc(hidden)]
    pub static ref APP_BUILDER_FN: Mutex<Option<Box<dyn Fn(&mut App) + Send>>> = Mutex::new(None);
}

#[derive(GodotClass, Default)]
#[class(base=Node)]
pub struct BevyApp {
    app: Option<App>,
}

impl BevyApp {
    pub fn get_app(&self) -> Option<&App> {
        self.app.as_ref()
    }

    pub fn get_app_mut(&mut self) -> Option<&mut App> {
        self.app.as_mut()
    }

    pub fn singleton() -> Gd<Self> {
        Engine::singleton()
            .get_main_loop()
            .unwrap()
            .cast::<SceneTree>()
            .get_root()
            .unwrap()
            .get_node_as::<Self>("BevyAppSingleton")
    }

    // pub fn get_app_from_base_mut<T: GodotClass>(base: BaseRef<T>) -> Option<&mut App>
    // where
    //     <T as GodotClass>::Base: Inherits<Node>,
    // {
    //     base.clone()
    //         .upcast::<Node>()
    //         .get_node_as::<Self>("/root/BevyAppSingleton")

    // }
}

#[godot_api]
impl INode for BevyApp {
    fn init(_base: Base<Node>) -> Self {
        Default::default()
    }

    fn ready(&mut self) {
        if godot::classes::Engine::singleton().is_editor_hint() {
            return;
        }

        let mut app = App::new();
        (APP_BUILDER_FN.lock().unwrap().as_mut().unwrap())(&mut app);
        app.add_plugins(bevy::app::TaskPoolPlugin::default())
            .add_plugins(bevy::log::LogPlugin::default())
            .add_plugins(bevy::diagnostic::FrameCountPlugin)
            .add_plugins(bevy::diagnostic::DiagnosticsPlugin)
            .add_plugins(bevy::time::TimePlugin)
            .add_plugins(crate::scene::PackedScenePlugin)
            .init_non_send_resource::<crate::scene_tree::SceneTreeRefImpl>();
        // .add_plugins(GodotSignalsPlugin)
        // .add_plugins(GodotInputEventPlugin);

        #[cfg(feature = "assets")]
        app.add_plugins(crate::assets::GodotAssetsPlugin);

        self.app = Some(app);
    }

    fn process(&mut self, _delta: f64) {
        if godot::classes::Engine::singleton().is_editor_hint() {
            return;
        }

        if let Some(app) = self.app.as_mut() {
            app.insert_resource(GodotVisualFrame);

            if let Err(e) = catch_unwind(AssertUnwindSafe(|| app.update())) {
                self.app = None;

                eprintln!("bevy app update panicked");
                resume_unwind(e);
            }

            app.world_mut().remove_resource::<GodotVisualFrame>();
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        if godot::classes::Engine::singleton().is_editor_hint() {
            return;
        }

        if let Some(app) = self.app.as_mut() {
            app.insert_resource(GodotPhysicsFrame);

            if let Err(e) = catch_unwind(AssertUnwindSafe(|| app.update())) {
                self.app = None;

                eprintln!("bevy app update panicked");
                resume_unwind(e);
            }

            app.world_mut().remove_resource::<GodotPhysicsFrame>();
        }
    }
}
