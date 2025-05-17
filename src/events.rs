use crate::BevyApp;
use bevy::ecs::event::Event;
use bevy_godot4_proc_macros::signal_event;
use godot::{
    global::godot_print,
    meta::{InParamTuple, ParamTuple},
    obj::{Gd, GodotClass, WithSignals},
    register::{SignalReceiver, TypedSignal},
};

struct EventSignalReciever<Ps: ParamTuple> {
    func: Box<dyn Fn(Ps)>,
}

impl<Ps> SignalReceiver<(), Ps> for EventSignalReciever<Ps>
where
    Ps: ParamTuple + 'static,
{
    fn call(&mut self, _instance: (), params: Ps) {
        self.func.call((params,));
    }
}

trait Instance<Ps, C, E>: Sized
where
    Ps: ParamTuple + InParamTuple + 'static,
    C: WithSignals + GodotClass,
    E: Event + From<(Self, Ps)>,
{
    fn from_instance(instance: Option<Gd<C>>) -> Self;
}

impl<Ps, C, E> Instance<Ps, C, E> for ()
where
    Ps: ParamTuple + InParamTuple + 'static,
    C: WithSignals + GodotClass,
    E: Event + From<(Self, Ps)>,
{
    fn from_instance(instance: Option<Gd<C>>) -> Self {
        if instance.is_some() {
            panic!("tried to convert gd instance to empty instance")
        }
        
    }
}

impl<Ps, C, E> Instance<Ps, C, E> for Gd<C>
where
    Ps: ParamTuple + InParamTuple + 'static,
    C: WithSignals + GodotClass,
    E: Event + From<(Self, Ps)>,
{
    fn from_instance(instance: Option<Gd<C>>) -> Self {
        let Some(value) = instance else {
            panic!("tried to convert empty instance into a gd");
        };
        value
    }
}

fn send_event<'a, E, Ps, I, C>(instance: I, params: Ps)
where
    Ps: ParamTuple + InParamTuple + 'static,
    C: WithSignals,
    I: Instance<Ps, C, E>,
    E: Event + From<(I, Ps)>,
{
    let x = BevyApp::singleton()
        .bind_mut()
        .get_app_mut()
        .unwrap()
        .world_mut()
        .send_event::<E>(E::from((instance, params)));
    godot_print!("{x:?}");
}

impl<'c> BevyApp {
    pub fn add_event<E, Ps, C>(&mut self, signal: &mut TypedSignal<'c, C, Ps>)
    where
        Ps: ParamTuple + InParamTuple + 'static,
        C: WithSignals,
        E: Event + From<((), Ps)>,
    {
        godot_print!("init event");

        signal.connect(EventSignalReciever {
            func: Box::new(move |params| send_event::<E, Ps, (), C>((), params)),
        });
    }

    pub fn add_event_instanced<E, Ps, C>(
        &mut self,
        instance: Gd<C>,
        signal: &mut TypedSignal<'c, C, Ps>,
    ) where
        Ps: ParamTuple + InParamTuple + 'static,
        C: WithSignals,
        E: Event + From<(Gd<C>, Ps)>,
    {
        signal.connect(EventSignalReciever {
            func: (Box::new(move |params| send_event::<E, Ps, Gd<C>, C>(instance.clone(), params))),
        });
    }
}

signal_event!(SignalEventEmpty);
