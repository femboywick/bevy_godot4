use crate::BevyApp;
use bevy::ecs::event::Event;
use bevy_godot4_proc_macros::signal_event;
use godot::{
    global::godot_print,
    meta::{InParamTuple, ParamTuple},
    obj::{Bounds, Gd, GodotClass, WithBaseField, WithSignals, bounds::DeclUser},
    register::{SignalReceiver, TypedSignal},
};

// TODO: write a macro to define events (mainly their from trait ? or find an alternative.)

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

// enum InstanceType<'a, C: WithSignals> {
//     Instanceless,
//     Instance(&'a C),
//     InstanceMut(&'a mut C),
// }

// trait FromInstanceType<T> {
//     fn from(value: T) -> Self;
// }

// trait Instance<'a, Ps, C, E>: FromInstanceType<InstanceType<'a, C>> + Sized
// where
//     Ps: ParamTuple + InParamTuple,
//     C: WithSignals,
//     E: Event + for<'c> From<(Self, Ps)>,
// {
//     fn to_func<'b: 'static>(
//         &self,
//         func: Box<dyn Fn(InstanceType<'b, C>, Ps)>,
//     ) -> EventSignalFunc<Ps, C>;
// }

// impl<'a, C> FromInstanceType<InstanceType<'a, C>> for ()
// where
//     C: WithSignals,
// {
//     fn from(_value: InstanceType<'a, C>) -> Self {
//         ()
//     }
// }

// impl<'a, C> FromInstanceType<InstanceType<'a, C>> for &'a C
// where
//     C: WithSignals,
// {
//     fn from(value: InstanceType<'a, C>) -> &'a C {
//         let InstanceType::Instance(c) = value else {
//             panic!("failed to convert instance into an immutable reference")
//         };
//         c
//     }
// }

// impl<'a, C> FromInstanceType<InstanceType<'a, C>> for &'a mut C
// where
//     C: WithSignals,
// {
//     fn from(value: InstanceType<'a, C>) -> &'a mut C {
//         let InstanceType::InstanceMut(c) = value else {
//             panic!("failed to convert instance into a mutable reference")
//         };
//         c
//     }
// }

// impl<'a, Ps, C, E> Instance<'a, Ps, C, E> for ()
// where
//     Ps: ParamTuple + InParamTuple + 'static,
//     C: WithSignals,
//     E: Event + for<'c> From<(Self, Ps)>,
// {
//     fn to_func<'b: 'static>(
//         &self,
//         func: Box<dyn Fn(InstanceType<'b, C>, Ps)>,
//     ) -> EventSignalFunc<Ps, C> {
//         EventSignalFunc::Instanceless(Box::new(move |ps: Ps| {
//             func.call((InstanceType::Instanceless, ps));
//         }))
//     }
// }

// fn instance_func<'b, Ps, C, E>(instance: &'b C, params: Ps)
// where
//     Ps: ParamTuple + InParamTuple + 'static,
//     C: WithSignals,
//     E: Event + for<'a> From<(&'a C, Ps)>,
// {
//     to_event::<E, Ps, &C, C>(InstanceType::Instance(instance), params);
// }

// impl<'a, Ps, C, E> Instance<'a, Ps, C, E> for &'a C
// where
//     Ps: ParamTuple + InParamTuple + 'static,
//     C: WithSignals,
//     E: Event + for<'c> From<(&'c C, Ps)>,
// {
//     fn to_func<'b: 'static>(
//         &self,
//         func: Box<dyn Fn(InstanceType<'b, C>, Ps)>,
//     ) -> EventSignalFunc<Ps, C> {
//         EventSignalFunc::Instance(Box::new(instance_func::<Ps, C, E>));
//         todo!()
//     }
// }

// fn instance_mut_func<'b, Ps, C, E>(instance: &'b mut C, params: Ps)
// where
//     Ps: ParamTuple + InParamTuple + 'static,
//     C: WithSignals,
//     E: Event + for<'a> From<(&'a mut C, Ps)>,
// {
//     to_event::<E, Ps, &mut C, C>(InstanceType::InstanceMut(instance), params);
// }

// impl<'a, Ps, C, E> Instance<'a, Ps, C, E> for &'a mut C
// where
//     Ps: ParamTuple + InParamTuple + 'static,
//     C: WithSignals,
//     E: Event + for<'c> From<(&'c mut C, Ps)>,
// {
//     fn to_func<'b: 'static>(
//         &self,
//         func: Box<dyn Fn(InstanceType<'b, C>, Ps)>,
//     ) -> EventSignalFunc<Ps, C> {
//         EventSignalFunc::InstanceMut(Box::new(instance_mut_func::<Ps, C, E>))
//     }
// }

trait Instance<Ps, C, E>: Sized
where
    Ps: ParamTuple + InParamTuple + 'static,
    C: WithSignals + GodotClass,
    E: Event + From<(Self, Ps)>,
{
    fn from_instance(instance: Option<Gd<C>>) -> Self;
}

// trait Instance<Ps, C, E>: Sized + FromInstance<Ps, C, E>
// where
//     Ps: ParamTuple + InParamTuple + 'static,
//     C: WithSignals + GodotClass,
//     E: Event + From<(Self, Ps)>,
// {
//     fn to_func(&mut self, func: Box<dyn Fn(Self, Ps)>) -> EventSignalFunc<Ps>;
// }

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
        ()
    }
}

// impl<Ps, C, E> Instance<Ps, C, E> for ()
// where
//     Ps: ParamTuple + InParamTuple + 'static,
//     C: WithSignals + GodotClass,
//     E: Event + From<(Self, Ps)>,
// {
//     fn to_func(&mut self, func: Box<dyn Fn(Self, Ps)>) -> EventSignalFunc<Ps> {
//         EventSignalFunc::Instanceless(Box::new(move |params| func.call(((), params))))
//     }
// }

// impl<Ps, C, E> Instance<Ps, C, E> for Gd<C>
// where
//     Ps: ParamTuple + InParamTuple + 'static,
//     C: WithSignals + GodotClass,
//     E: Event + From<(Self, Ps)>,
// {
//     fn to_func(&mut self, func: Box<dyn Fn(Self, Ps)>) -> EventSignalFunc<Ps> {
//         EventSignalFunc::Instance(Box::new(move |instance, params| {
//             func.call((instance, params))
//         }))
//     }
// }

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
        mut instance: Gd<C>,
        signal: &mut TypedSignal<'c, C, Ps>,
    ) where
        Ps: ParamTuple + InParamTuple + 'static,
        C: WithSignals,
        E: Event + From<(Gd<C>, Ps)>,
    {
        signal
            // .connect_builder()
            // .object_self()
            .connect(EventSignalReciever {
                func: (Box::new(move |params| {
                    send_event::<E, Ps, Gd<C>, C>(instance.clone(), params)
                })),
            });
    }
}

signal_event!(SignalEventEmpty);
