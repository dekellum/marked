#![warn(rust_2018_idioms)]

#[macro_use] extern crate html5ever;

pub mod vdom;
pub mod decode;
mod chars;

#[cfg(test)]
mod logger;

#[macro_use] mod macros;
