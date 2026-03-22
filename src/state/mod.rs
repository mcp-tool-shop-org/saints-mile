//! State system — what the game remembers across scenes, encounters, and chapters.
//!
//! The state contract: relay branch is first-class, reputation is a web,
//! evidence has integrity, witness states track alive/location/integrity/testified,
//! memory objects are explicit, the hand injury is a first-class field.

pub mod types;
pub mod store;
pub mod argument;
pub mod evidence;
pub mod investigation;
pub mod history;
