use bevy::prelude::{Entity, World};
use crate::prelude::{RxRef, RxVal};
use crate::RxApply;

pub trait RxWorldExt<T> {
    fn spawn_rx(&mut self, rx_bundle: impl RxApply<T>) -> RxRef<T>;
    fn insert_rx(&mut self, entity: Entity, rx: impl RxApply<T>) -> RxRef<T>;
}

impl <T: RxVal> RxWorldExt<T> for World {
    fn spawn_rx(&mut self, rx: impl RxApply<T>) -> RxRef<T> {
        rx.apply(self)
    }

    fn insert_rx(&mut self, entity: Entity, rx: impl RxApply<T>) -> RxRef<T> {
        rx.insert(self, entity)
    }
}
