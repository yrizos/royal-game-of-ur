//! `ur-core` — pure game logic for the Royal Game of Ur.
//!
//! This crate contains no I/O, no rendering, and no platform-specific code.
//! All state is passed explicitly; all randomness is caller-provided.

pub mod ai;
pub mod board;
pub mod dice;
pub mod player;
pub mod state;
