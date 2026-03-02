//! Shared request/response types.

pub mod common;
pub mod computers;
pub mod jobs;
pub mod people;
pub mod queue;
pub mod system;
pub mod users;
pub mod views;

pub use common::*;
pub use computers::*;
pub use jobs::*;
pub use people::*;
pub use queue::*;
pub use system::*;
pub use users::*;
pub use views::*;
