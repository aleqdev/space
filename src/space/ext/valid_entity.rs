pub trait EntityOpsExt {
    fn is_valid(&self) -> bool;
    fn invalidate(&mut self);
}

impl EntityOpsExt for bevy::prelude::Entity {
    fn is_valid(&self) -> bool {
        self.index() != u32::MAX
    }

    fn invalidate(&mut self) {
        *self = bevy::prelude::Entity::from_raw(u32::MAX);
    }
}
