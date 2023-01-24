use bevy::prelude::*;
use ringbuffer;

#[derive(Debug, Default, Component, derive_more::From, derive_more::Into)]
pub struct BodyTrail {
    pub buffer: ringbuffer::AllocRingBuffer<Vec3>
}
