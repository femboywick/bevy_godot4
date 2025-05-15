use crate::BevyApp;
use bevy::ecs::event::Event;
use godot::{
    global::godot_print,
    meta::{InParamTuple, OutParamTuple, ParamTuple},
    obj::WithSignals,
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

impl<'c> BevyApp {
    pub fn add_event<C, Ps, E>(&mut self, signal: &mut TypedSignal<'c, C, Ps>)
    where
        C: WithSignals,
        Ps: ParamTuple + OutParamTuple + InParamTuple + 'static,
        E: Event + From<Ps>,
    {
        self.get_app_mut().unwrap().add_event::<E>();

        signal.connect(EventSignalReciever {
            func: Box::new(|params| {
                BevyApp::singleton()
                    .bind_mut()
                    .get_app_mut()
                    .unwrap()
                    .world_mut()
                    .send_event::<E>(E::from(params));
            }),
        });
    }
}

#[derive(bevy::prelude::Event)]
pub struct SignalEventEmpty;

impl From<()> for SignalEventEmpty {
    fn from(_value: ()) -> Self {
        godot_print!("from ()");
        Self
    }
}

macro_rules! signal_event {
    ($name:ident, $($p_name:ident, $p_type:ty),+) => {
        #[derive(bevy::prelude::Event)]
        pub struct $name {
            $($p_name: $p_type),+
        }

        pub impl From<($($p_type),+)> for $name {
            fn from()
        }
    };
}
