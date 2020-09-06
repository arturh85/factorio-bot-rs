#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate evmap_derive;
#[macro_use]
extern crate paris;
#[macro_use]
extern crate enum_primitive_derive;
#[macro_use]
extern crate async_trait;
extern crate num_traits;
extern crate strum;
#[macro_use]
extern crate strum_macros;

pub mod factorio;
pub mod types;
pub mod web;
