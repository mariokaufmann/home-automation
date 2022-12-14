#![deny(clippy::all)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

pub mod action;
pub mod automacro;
pub mod automodule;
pub mod config;
pub mod fs;
pub mod test;
pub mod websocket;
