pub mod bot;
pub mod config;
mod protocol;
mod runtime;
pub mod search;
pub mod tetris;

pub use runtime::dispatcher::run;
