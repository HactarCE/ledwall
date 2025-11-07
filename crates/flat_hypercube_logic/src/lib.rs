//! Flat Hypercube puzzle logic.
//!
//! This crate only supports simulation of 3^4.

mod float_geom;
mod geom;
mod puzzle;

pub use float_geom::*;
pub use geom::*;
pub use puzzle::*;

pub const SCRAMBLE_MOVE_COUNT: usize = 5000;
