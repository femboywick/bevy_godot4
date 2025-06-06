use bevy::{
    app::{App, FixedMain, Main},
    time::{Fixed, Time},
};
use godot::{
    classes::{Engine, INode, Node, SceneTree},
    obj::{Base, Gd},
    prelude::{GodotClass, godot_api},
};

use std::{
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    sync::Mutex,
};

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

    pub fn app(&self) -> &App {
        self.get_app().unwrap()
    }

    pub fn app_mut(&mut self) -> &mut App {
        self.get_app_mut().unwrap()
    }

    pub fn singleton() -> Gd<Self> {
        Self::get_singleton().expect("could not get BevyAppSingleton")
    }

    pub fn get_singleton() -> Option<Gd<Self>> {
        Some(
            Engine::singleton()
                .get_main_loop()?
                .cast::<SceneTree>()
                .get_root()?
                .get_node_as::<Self>("BevyAppSingleton"),
        )
    }
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
        app.add_plugins(bevy::app::TaskPoolPlugin::default())
            .add_plugins(bevy::log::LogPlugin::default())
            .add_plugins(bevy::diagnostic::FrameCountPlugin)
            .add_plugins(bevy::diagnostic::DiagnosticsPlugin)
            .add_plugins(bevy::time::TimePlugin)
            .add_plugins(crate::scene::PackedScenePlugin)
            .init_non_send_resource::<crate::scene_tree::SceneTreeRefImpl>()
            .insert_resource(Time::<Fixed>::from_hz(60.));

        (APP_BUILDER_FN.lock().unwrap().as_mut().unwrap())(&mut app);

        #[cfg(feature = "assets")]
        app.add_plugins(crate::assets::GodotAssetsPlugin);

        app.update();

        self.app = Some(app);
    }

    fn process(&mut self, _delta: f64) {
        if godot::classes::Engine::singleton().is_editor_hint() {
            return;
        }

        if let Some(app) = self.app.as_mut()
            && let Err(e) = catch_unwind(AssertUnwindSafe(|| app.world_mut().run_schedule(Main)))
        {
            self.app = None;

            eprintln!("bevy app update panicked");
            resume_unwind(e);
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        if godot::classes::Engine::singleton().is_editor_hint() {
            return;
        }

        if let Some(app) = self.app.as_mut()
            && let Err(e) =
                catch_unwind(AssertUnwindSafe(|| app.world_mut().run_schedule(FixedMain)))
        {
            self.app = None;

            eprintln!("bevy app update panicked");
            resume_unwind(e);
        }
    }
}
