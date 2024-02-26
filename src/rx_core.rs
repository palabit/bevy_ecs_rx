use std::fmt::Debug;
use std::marker::PhantomData;

use bevy::prelude::{Entity, World};
use crate::components::{RxEffect, RxSignal};
use crate::RxApply;

pub trait RxVal: Sized + Copy + Send + Sync + 'static {}

impl <T: Sized + Copy + Send + Sync + 'static> RxVal for T {}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct RxRef<T> {
    pub entity: Entity,
    _phantom: PhantomData<T>,
}

impl <T> RxRef<T> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _phantom: PhantomData::default(),
        }
    }
}

pub struct RxValue<T: RxVal> {
    pub value: T
}

impl <T: RxVal> RxValue<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl <T: RxVal> RxApply<T> for RxValue<T> {
    fn insert(self, world: &mut World, entity: Entity) -> RxRef<T> {
        world.entity_mut(entity).insert(RxSignal::new(self.value));
        RxRef::new(entity)
    }
}

pub struct RxMap<A: RxVal, B: RxVal, F: Fn(&A) -> B + Send + Sync + 'static> {
    pub source: RxRef<A>,
    pub mapping: F
}

impl <A: RxVal, B: RxVal, F: Fn(&A) -> B + Send + Sync + 'static> RxMap<A, B, F> {
    pub fn new(source: RxRef<A>, mapping: F) -> Self {
        Self {
            source,
            mapping,
        }
    }
}

impl <A: Debug + RxVal, B: Debug + Default + RxVal, F: Fn(&A) -> B + Send + Sync + 'static> RxApply<B> for RxMap<A, B, F> {
    fn insert(self, world: &mut World, destination: Entity) -> RxRef<B> {
        let source = self.source.entity;
        world.entity_mut(destination).insert(RxSignal::new(B::default()));
        let mapping = self.mapping;
        let system = move |world: &mut World| {
            let Some(r) = world.entity(source)
                .get::<RxSignal>()
                .and_then(|signal| signal.get())
                else {
                    return;
                };
            let mapped = mapping(r);
            let mut binding = world.entity_mut(destination);
            if let Some(mut signal) = binding.get_mut::<RxSignal>() {
                signal.set(mapped);
            }
        };
        let system_id = world.register_system(system);
        world.entity_mut(destination).insert(RxEffect::new(source, system_id));

        RxRef::new(destination)
    }
}

pub struct RxConsumer<T: RxVal, F: Fn(&T, &mut World, Entity) + Send + Sync + 'static> {
    pub source: RxRef<T>,
    pub effect: F
}

impl <T: RxVal, F: Fn(&T, &mut World, Entity) + Send + Sync + 'static> RxConsumer<T, F> {
    pub fn new(source: RxRef<T>, effect: F) -> Self {
        Self {
            source,
            effect
        }
    }
}

impl <T: RxVal + Clone + Debug, F: Fn(&T, &mut World, Entity) + Send + Sync + 'static> RxApply<()> for RxConsumer<T, F> {
    fn insert(self, world: &mut World, destination: Entity) -> RxRef<()> {
        let source = self.source.entity;
        let effect = self.effect;
        let system = move |world: &mut World| {
            let Some(r) = world.entity(source)
                .get::<RxSignal>()
                .and_then(|signal| signal.get::<T>())
                else {
                    return;
                };
            let r = r.clone();
            effect(&r, world, destination);
        };
        let system_id = world.register_system(system);
        world.entity_mut(destination).insert(RxEffect::new(source, system_id));

        RxRef::new(destination)
    }
}
