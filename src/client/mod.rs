//! Client implementations (async + optional blocking).

#[cfg(feature = "async")]
pub mod async_client;
#[cfg(feature = "blocking")]
pub mod blocking_client;

#[cfg(feature = "async")]
pub use async_client::{Client, ClientBuilder};
#[cfg(feature = "blocking")]
pub use blocking_client::{BlockingClient, BlockingClientBuilder};
