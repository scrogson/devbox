#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
extern crate colored;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate prettytable;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tempdir;
extern crate toml;

mod cmd;
mod errors;
mod project;
mod service;
mod task;

pub mod cli;

pub use cmd::*;
pub use errors::*;
pub use project::*;
pub use service::*;
