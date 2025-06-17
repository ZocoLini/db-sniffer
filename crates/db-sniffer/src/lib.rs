#![allow(unused)]

extern crate core;

use getset::Getters;
use std::str::FromStr;

mod db_objects;
mod sniffers;
mod error;
mod naming;

pub mod generators;

pub use db_objects::Table;

pub use error::Error;
pub use sniffers::sniff;
pub use sniffers::SniffResults;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");