use std::fmt::Debug;

use bevy::prelude::*;
use crate::prelude::{RxComputed, RxEffect, RxSignal};
use crate::rx_core::RxRef;

pub mod rx_core;
pub mod world;
pub mod components;

pub mod prelude {
    pub use crate::rx_core::*;
    pub use crate::world::*;
    pub use crate::components::*;
    pub use crate::{RxPlugin, RxUpdate, RxApply};
}

pub struct RxPlugin;

impl Plugin for RxPlugin {
    fn build(&self, app: &mut App) {
        app
            .configure_sets(Update, RxUpdate)
            .add_systems(Update, run_rx.in_set(RxUpdate))
        ;
    }
}

#[derive(SystemSet, Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RxUpdate;

pub trait RxApply<T>: Sized {
    fn apply(self, world: &mut World) -> RxRef<T> {
        let destination = world.spawn_empty().id();
        self.insert(world, destination)
    }

    fn insert(self, world: &mut World, entity: Entity) -> RxRef<T>;
}

fn run_rx(world: &mut World) {
    let mut updated: Vec<_> = world.query_filtered::<Entity, (Changed<RxSignal>, Without<RxComputed>)>().iter(world).collect();
    let effects: Vec<_> = world.query::<(Entity, &RxEffect)>().iter(world).map(|(entity, effect)| (entity, effect.depends_on, effect.effect)).collect();

    let mut new_updated = Vec::new();
    loop {
        for (entity, source, system_id) in &effects {
            if updated.contains(source) {
                world.run_system(*system_id).expect("Running system failed");
                new_updated.push(*entity);
            }
        }
        if new_updated.is_empty() {
            break;
        }
        (updated, new_updated) = (new_updated, updated);
        new_updated.clear();
    }

}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use super::*;

    #[derive(Component)]
    struct Text(String);

    #[test]
    fn create_and_update_signal() {
        let mut app = App::new();
        app.add_plugins(RxPlugin);

        let counter = app.world.spawn_rx(RxValue::new(1_u32));
        let double_counter = app.world.spawn_rx(RxMap::new(counter, |x| *x * 2));
        let add_counter = app.world.spawn_rx(RxMap::new(double_counter, |x| *x + 10));
        let consume = app.world.spawn_rx(RxConsumer::new(add_counter, |x, world, self_entity| {
            if let Some(mut text) = world.entity_mut(self_entity).get_mut::<Text>() {
                text.0 = format!("x={}", x);
            }
        }));
        app.world.entity_mut(consume.entity).insert(Text(String::default()));

        app.update();

        let actual = &app.world.entity(consume.entity).get::<Text>().unwrap().0;

        assert_eq!("x=12", actual);
    }
}
