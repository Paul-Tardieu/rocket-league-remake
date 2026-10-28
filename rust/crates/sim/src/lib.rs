//! Deterministic, headless car-soccer physics.
//!
//! Build a [`World`], feed it one [`CarInput`] per car each tick with [`World::step`], and read the
//! result with [`World::state`]. Nothing here renders, reads the clock or uses unseeded randomness.

mod arena;
mod ball;
mod car;
mod config;
mod contact;
mod state;
mod world;

pub use config::{ArenaSpec, BallSpec, BoostPad, BoostSpec, CarSpec, PhysicsConfig};
pub use glam::{DQuat, DVec2, DVec3};
pub use state::{BallState, CarInput, CarState, GameEvent, GameState, Score, Team};
pub use world::World;
