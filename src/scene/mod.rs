//! Scene system — dialogue, choices, conditions, state effects, memory callbacks.
//!
//! The scene contract: every scene has speakers, text, choices, conditions,
//! state effects, pacing tags, and memory callbacks. Memory objects (biscuit cloth,
//! flask, poster) are explicitly tagged so they echo later.

pub mod types;
