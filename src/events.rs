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

trait Instance {}

impl Instance for () {}

impl<C: GodotClass> Instance for Gd<C> {}

fn send_event<E, Ps, T>(instance: T, params: Ps)
where
    Ps: ParamTuple + InParamTuple + 'static,
    T: Instance,
    E: Event + From<(T, Ps)>,
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
            func: Box::new(move |params| send_event::<E, Ps, ()>((), params)),
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
            func: (Box::new(move |params| send_event::<E, Ps, Gd<C>>(instance.clone(), params))),
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
            func: (Box::new(move |params| send_event::<E, Ps, Gd<T>>(instance.clone(), params))),
        });
    }
}

signal_event!(SignalEventEmpty);
