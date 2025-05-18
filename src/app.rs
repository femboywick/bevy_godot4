use bevy::prelude::IntoScheduleConfigs;
use bevy::{
    app::{App, First, FixedMain, FixedMainScheduleOrder, Main, MainScheduleOrder, Plugin},
    ecs::{
        component::Tick,
        event::{EventRegistry, ShouldUpdateEvents},
        hierarchy::{ChildOf, Children},
        name::Name,
        reflect::AppTypeRegistry,
        schedule::{ExecutorKind, Schedule, ScheduleLabel},
        system::Local,
        world::{Mut, World},
    },
    time::{Fixed, Time},
};
use godot::{
    classes::{Engine, INode, Node, SceneTree},
    global::godot_print,
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

#[derive(ScheduleLabel, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PhysicsProcess;

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
        // app.sub_apps_mut().main.update_schedule = Some(Main.intern());

        // app.init_resource::<AppTypeRegistry>()
        //     .register_type::<Name>()
        //     .register_type::<ChildOf>()
        //     .register_type::<Children>()
        //     .add_plugins(GodotSchedulePlugin);

        // app.add_systems(
        //     First,
        //     event_update_system
        //         .in_set(bevy::ecs::event::EventUpdates)
        //         .run_if(bevy::ecs::event::event_update_condition),
        // );

        app.add_plugins(bevy::app::TaskPoolPlugin::default())
            .add_plugins(bevy::log::LogPlugin::default())
            .add_plugins(bevy::diagnostic::FrameCountPlugin)
            .add_plugins(bevy::diagnostic::DiagnosticsPlugin)
            .add_plugins(bevy::time::TimePlugin)
            .add_plugins(crate::scene::PackedScenePlugin)
            .init_non_send_resource::<crate::scene_tree::SceneTreeRefImpl>()
            // .add_schedule(Schedule::new(PhysicsProcess));
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

        if let Some(app) = self.app.as_mut() {
            // app.insert_resource(GodotVisualFrame);

            if let Err(e) = catch_unwind(AssertUnwindSafe(|| app.world_mut().run_schedule(Main))) {
                self.app = None;

                eprintln!("bevy app update panicked");
                resume_unwind(e);
            }

            // app.world_mut().remove_resource::<GodotVisualFrame>();
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        if godot::classes::Engine::singleton().is_editor_hint() {
            return;
        }

        if let Some(app) = self.app.as_mut() {
            // app.insert_resource(GodotPhysicsFrame);

            if let Err(e) =
                catch_unwind(AssertUnwindSafe(|| app.world_mut().run_schedule(FixedMain)))
            {
                self.app = None;

                eprintln!("bevy app update panicked");
                resume_unwind(e);
            }

            // app.world_mut().remove_resource::<GodotPhysicsFrame>();
        }
    }
}

// struct GodotSchedulePlugin;

// impl Plugin for GodotSchedulePlugin {
//     fn build(&self, app: &mut App) {
//         let mut main_schedule = Schedule::new(Main);
//         main_schedule.set_executor_kind(ExecutorKind::SingleThreaded);
//         let mut fixed_main_schedule = Schedule::new(FixedMain);
//         fixed_main_schedule.set_executor_kind(ExecutorKind::SingleThreaded);

//         app.add_schedule(main_schedule)
//             .add_schedule(fixed_main_schedule)
//             .init_resource::<MainScheduleOrder>()
//             .init_resource::<FixedMainScheduleOrder>()
//             .add_systems(Main, Main::run_main)
//             .add_systems(FixedMain, FixedMain::run_fixed_main);
//     }
// }

// pub fn event_update_system(world: &mut World, mut last_change_tick: Local<Tick>) {
//     if world.contains_resource::<EventRegistry>() {
//         world.resource_scope(|world, mut registry: Mut<EventRegistry>| {
//             registry.run_updates(world, *last_change_tick);

//             registry.should_update = match registry.should_update {
//                 // If we're always updating, keep doing so.
//                 ShouldUpdateEvents::Always => ShouldUpdateEvents::Always,
//                 // Disable the system until signal_event_update_system runs again.
//                 ShouldUpdateEvents::Waiting | ShouldUpdateEvents::Ready => {
//                     ShouldUpdateEvents::Waiting
//                 }
//             };
//         });
//     }
//     *last_change_tick = world.change_tick();
// }
