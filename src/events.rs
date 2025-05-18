use crate::BevyApp;
use bevy::ecs::event::Event;
use bevy_godot4_proc_macros::signal_event;
use godot::{
    global::godot_print,
    meta::{InParamTuple, ParamTuple},
    obj::{Bounds, Gd, GodotClass, WithSignals, bounds::DeclUser},
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
}

impl<Ps, C, E> Instance<Ps, C, E> for ()
where
    Ps: ParamTuple + InParamTuple + 'static,
    C: WithSignals + GodotClass,
    E: Event + From<(Self, Ps)>,
{
}

impl<Ps, C, E> Instance<Ps, C, E> for Gd<C>
where
    Ps: ParamTuple + InParamTuple + 'static,
    C: WithSignals + GodotClass,
    E: Event + From<(Self, Ps)>,
{
}

fn send_event<'a, E, Ps, I, C>(instance: I, params: Ps)
where
    Ps: ParamTuple + InParamTuple + 'static,
    C: WithSignals,
    I: Instance<Ps, C, E>,
    E: Event + From<(I, Ps)>,
{
    let mut event = E::from((instance, params));
    BevyApp::singleton()
        .bind_mut()
        .app_mut()
        .world_mut()
        .trigger_ref(&mut event);
    BevyApp::singleton()
        .bind_mut()
        .app_mut()
        .world_mut()
        .send_event::<E>(event);
}

fn send_event_obj<'a, E, Ps, T, C>(instance: Gd<T>, params: Ps)
where
    Ps: ParamTuple + InParamTuple + 'static,
    C: WithSignals,
    T: GodotClass,
    E: Event + From<(Gd<T>, Ps)>,
{
    let mut event = E::from((instance, params));
    BevyApp::singleton()
        .bind_mut()
        .app_mut()
        .world_mut()
        .trigger_ref(&mut event);
    BevyApp::singleton()
        .bind_mut()
        .app_mut()
        .world_mut()
        .send_event::<E>(event);
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

    pub fn add_event_obj<E, Ps, C, T>(
        &mut self,
        instance: Gd<T>,
        signal: &mut TypedSignal<'c, C, Ps>,
    ) where
        Ps: ParamTuple + InParamTuple + 'static,
        C: WithSignals,
        E: Event + From<(Gd<T>, Ps)>,
        T: GodotClass + Bounds<Declarer = DeclUser>,
    {
        signal.connect(EventSignalReciever {
            func: (Box::new(move |params| send_event_obj::<E, Ps, T, C>(instance.clone(), params))),
        });
    }
}

signal_event!(SignalEventEmpty);
