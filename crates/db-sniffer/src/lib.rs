#![allow(unused)]

extern crate core;

use getset::Getters;
use std::str::FromStr;

mod db_objects;
mod sniffers;
mod error;
mod naming;

pub mod generators;

pub use error::Error;
pub use sniffers::sniff;
