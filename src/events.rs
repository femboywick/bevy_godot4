use crate::BevyApp;
use bevy::ecs::event::Event;
use bevy_godot4_proc_macros::signal_event;
use godot::meta::FromGodot;
use godot::{
    global::godot_print,
    meta::{InParamTuple, ParamTuple},
    obj::{Bounds, Gd, GodotClass, WithSignals, bounds::DeclUser},
    register::TypedSignal,
};
use paste::*;
use std::fmt::Debug;

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

pub trait AddEvent<Ps: ParamTuple> {
    fn add_event<'c, E, C>(&mut self, signal: &mut TypedSignal<'c, C, Ps>)
    where
        C: WithSignals + GodotClass,
        E: Event + From<((), Ps)>;
    fn add_event_instanced<'c, E, C>(
        &mut self,
        instance: Gd<C>,
        signal: &mut TypedSignal<'c, C, Ps>,
    ) where
        C: WithSignals + GodotClass,
        E: Event + From<(Gd<C>, Ps)>;
    fn add_event_obj<'c, E, C, T>(&mut self, instance: Gd<T>, signal: &mut TypedSignal<'c, C, Ps>)
    where
        C: WithSignals + GodotClass,
        T: GodotClass + Bounds<Declarer = DeclUser>,
        E: Event + From<(Gd<T>, Ps)>;
}

macro_rules! impl_add_event {
    ($($($p:expr),+)?) => {
        paste!{
        impl<$($($p),+)?> AddEvent<($($($p),+,)?)> for  BevyApp where $($($p: Debug + FromGodot + 'static),+)? {
            fn add_event<'c, E, C, >(&mut self, signal: &mut TypedSignal<'c, C, ($($($p),+,)?)>)
            where
                // Ps: ParamTuple + InParamTuple + 'static,
                C: WithSignals + GodotClass,
                $($($p: Debug + FromGodot + 'static),+,)?
                E: Event + From<((), ($($($p),+,)?))>,
            {
                godot_print!("init event");

                signal
                    .connect(|$($([<$p:lower>]),+)?| send_event::<E, ($($($p),+,)?), ()>((), ($($([<$p:lower>]),+,)?)));
            }

            fn add_event_instanced<'c, E, C>(
                &mut self,
                instance: Gd<C>,
                signal: &mut TypedSignal<'c, C, ($($($p),+,)?)>,
            ) where
                C: WithSignals,
                $($($p: Debug + FromGodot + 'static),+,)?
                E: Event + From<(Gd<C>, ($($($p),+,)?))>,
            {
                signal
                    .connect(move |$($([<$p:lower>]),+)?| send_event::<E, ($($($p),+,)?), Gd<C>>(instance.clone(), ($($([<$p:lower>]),+,)?)));
            }

            fn add_event_obj<'c, E, C, T>(
                &mut self,
                instance: Gd<T>,
                signal: &mut TypedSignal<'c, C, ($($($p),+,)?)>,
            ) where
                // Ps: ParamTuple + InParamTuple + 'static,
                C: WithSignals,
                E: Event + From<(Gd<T>, ($($($p),+,)?))>,
                T: GodotClass + Bounds<Declarer = DeclUser>,
            {
                signal
                    .connect(move |$($([<$p:lower>]),+)?| send_event::<E, ($($($p),+,)?), Gd<T>>(instance.clone(), ($($([<$p:lower>]),+,)?)));
            }
        }}
    };
}

impl_add_event!();
impl_add_event!(P0);
impl_add_event!(P0, P1);
impl_add_event!(P0, P1, P2);
impl_add_event!(P0, P1, P2, P3);
impl_add_event!(P0, P1, P2, P3, P4);
impl_add_event!(P0, P1, P2, P3, P4, P5);

signal_event!(SignalEventEmpty);
