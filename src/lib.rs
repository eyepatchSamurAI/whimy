#![deny(clippy::all)]

mod registries;
mod signatures;
mod wmi;
mod async_wmi;

#[macro_use]
extern crate napi_derive;
