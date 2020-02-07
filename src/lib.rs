#![recursion_limit = "1024"]

extern crate colored;
extern crate dirs;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate prettytable;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tempdir;
extern crate toml;

mod errors;
mod project;
mod service;
mod task;

pub use crate::errors::*;
pub use crate::project::*;
pub use crate::service::*;
