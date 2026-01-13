//! High-level Jenkins **core** API services.
//!
//! The primary SDK surface is exposed via service accessors on clients:
//! - `Client::jobs()` / `BlockingClient::jobs()`
//! - `Client::queue()` / `BlockingClient::queue()`
//! - `Client::system()` / `BlockingClient::system()`

pub mod computers;
pub mod jobs;
pub mod people;
pub mod queue;
pub mod system;
pub mod users;
pub mod views;

pub use computers::*;
pub use jobs::*;
pub use people::*;
pub use queue::*;
pub use system::*;
pub use users::*;
pub use views::*;
