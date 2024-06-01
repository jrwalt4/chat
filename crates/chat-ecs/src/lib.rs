//! An Entiy-Component-System tailored to
//! calculations performed in SWMMM engine.

#![allow(dead_code)]

mod component;
mod entity;
mod system;
mod world;

pub use entity::Entity;
pub use world::World;
