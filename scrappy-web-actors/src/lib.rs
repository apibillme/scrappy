#![allow(clippy::borrow_interior_mutable_const)]
//! scrappy actors integration for scrappy web framework
mod context;
pub mod ws;

pub use self::context::HttpContext;
