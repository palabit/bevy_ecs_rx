use std::any::Any;
use std::fmt::Debug;

use bevy::ecs::system::SystemId;
use bevy::prelude::{Component, Entity};

use crate::prelude::RxVal;

#[derive(Component)]
pub struct RxSignal {
    value: Box<dyn Any + Sync + Send>
}

impl RxSignal {
    pub fn new<T: RxVal>(value: T) -> Self {
        Self {
            value: Box::new(value)
        }
    }

    pub fn get<T: RxVal>(&self) -> Option<&T> {
        self.value.downcast_ref()
    }

    pub fn set<T: RxVal>(&mut self, value: T) -> Option<()> {
        self.value.downcast_mut().map(move |r| *r = value)
    }
}

#[derive(Component, Debug, Default)]
pub struct RxComputed;

#[derive(Component, Debug)]
pub struct RxEffect {
    pub depends_on: Entity,
    pub effect: SystemId,
}

impl RxEffect {
    pub fn new(depends_on: Entity, effect: SystemId) -> Self {
        Self {
            depends_on,
            effect
        }
    }
}
