use bevy::{
    ecs::{component::ComponentInfo, system::SystemParam},
    prelude::*,
};
use godot::{
    classes::{Node, Object},
    global::godot_print,
    obj::{AsDyn, Bounds, Inherits, bounds::DeclUser},
};
use std::{
    marker::PhantomData,
    time::{Duration, Instant},
};

use crate::{erased_gd::DynErasedGd, prelude::TypedErasedGd};

pub trait HasEntity {
    fn entity(&self) -> Entity {
        self.get_entity().expect("could not get entity")
    }
    fn get_entity(&self) -> Option<Entity>;
}

#[derive(Component)]
pub struct ComponentMarker {
    parent: DynErasedGd<dyn HasEntity>,
}

impl ComponentMarker {
    pub fn new(parent: DynErasedGd<dyn HasEntity>) -> Self {
        Self { parent }
    }
}

pub fn add_components<T: Inherits<Object> + HasEntity + AsDyn<dyn HasEntity>>(
    mut children: Query<(Entity, &mut ComponentMarker)>,
    mut commands: Commands,
) where
    T: Bounds<Declarer = DeclUser>,
{
    for (entity_id, mut child) in children.iter_mut() {
        let Some(target_gd) = child.parent.try_get::<T>() else {
            continue;
        };
        let target = target_gd.bind().entity();
        commands
            .entity(entity_id)
            .remove::<ComponentMarker>()
            .clone_with(target, |_| {})
            .despawn();

        commands.entity(target).queue(log_components);
        // .queue(log_components);
    }
}

pub fn log_components(entity: EntityWorldMut) {
    let debug_infos: Vec<_> = entity
        .world()
        .inspect_entity(entity.id())
        .expect("Entity existence is verified before an EntityCommand is executed")
        .map(ComponentInfo::name)
        .collect();
    godot_print!("Entity {}: {debug_infos:?}", entity.id());
}

/// SystemParam to keep track of an independent delta time
///
/// Not every system runs on a Bevy update and Bevy can be updated multiple
/// during a "frame".
#[derive(SystemParam)]
pub struct SystemDeltaTimer<'w, 's> {
    last_time: Local<'s, Option<Instant>>,
    marker: PhantomData<&'w ()>,
}

impl SystemDeltaTimer<'_, '_> {
    /// Returns the time passed since the last invocation
    pub fn delta(&mut self) -> Duration {
        let now = Instant::now();
        let last_time = self.last_time.unwrap_or(now);

        *self.last_time = Some(now);

        now - last_time
    }

    pub fn delta_seconds(&mut self) -> f32 {
        self.delta().as_secs_f32()
    }

    pub fn delta_seconds_f64(&mut self) -> f64 {
        self.delta().as_secs_f64()
    }
}

#[macro_export]
macro_rules! bevy_spawn {
    ($($component:expr),+) => {
        bevy_godot4::BevyApp::singleton()
            .bind_mut()
            .get_app_mut()
            .unwrap()
            .world_mut()
            .spawn(( $($component),+ ))
    };
}
