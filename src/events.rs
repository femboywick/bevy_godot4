use crate::BevyApp;
use bevy::ecs::event::Event;
use godot::{
    meta::{InParamTuple, OutParamTuple, ParamTuple},
    obj::WithSignals,
    prelude::*,
    register::{SignalReceiver, TypedSignal},
};

use crate::get_app_mut;

#[derive(GodotClass)]
#[class(init, base = Node)]
pub struct EventSender {
    base: Base<Node>,
}

impl<'c> EventSender {
    pub fn add_event<C: WithSignals, Ps, E: Event>(&mut self, signal: &mut TypedSignal<'c, C, Ps>)
    where
        Ps: ParamTuple + OutParamTuple + InParamTuple + 'static,
    {
        get_app_mut!(self.base()).add_event::<E>();

        let func = match Ps::LEN {
            0 => |obj: &mut EventSender| {},
            _ => panic!("too long"),
        };

        signal.connect_obj(&self.to_gd(), func);
    }
}
