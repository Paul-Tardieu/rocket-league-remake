//! Bots for carball. A bot turns the latest [`GameState`] into a [`CarInput`] for its car; so does
//! the keyboard, which makes humans and bots interchangeable for the game.

pub mod harness;
mod prediction;
mod rng;
mod striker;

pub use prediction::{Landing, landing};
pub use rng::SplitMix64;
pub use striker::StrikerBot;

use carball_sim::{CarInput, GameState};

/// Controls one car. Called once per simulation tick with the latest state.
pub trait Bot {
    fn act(&mut self, state: &GameState) -> CarInput;
}

impl<F: FnMut(&GameState) -> CarInput> Bot for F {
    fn act(&mut self, state: &GameState) -> CarInput {
        self(state)
    }
}
