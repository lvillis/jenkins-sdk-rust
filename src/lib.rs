//! Jenkins API SDK in pure Rust.
//!
//! ## Quick start (async)
//! ```no_run
//! # #[cfg(feature = "async")]
//! # async fn demo() -> Result<(), jenkins_sdk::Error> {
//! use jenkins_sdk::Client;
//! use std::time::Duration;
//!
//! let client = Client::builder("https://jenkins.example.com")?
//!     .auth_basic("user", "token")
//!     .with_retry(3, Duration::from_millis(200))
//!     .with_crumb(Duration::from_secs(1800))
//!     .build()?;
//!
//! let q: serde_json::Value = client.queue().list(None).await?;
//! println!("{q:?}");
//! # Ok(())
//! # }
//! ```
//!
//! ## Quick start (blocking)
//! Enable the `blocking` feature:
//! ```no_run
//! # #[cfg(feature = "blocking")]
//! # fn demo() -> Result<(), jenkins_sdk::Error> {
//! use jenkins_sdk::BlockingClient;
//! use std::time::Duration;
//!
//! let client = BlockingClient::builder("https://jenkins.example.com")?
//!     .auth_basic("user", "token")
//!     .with_retry(2, Duration::from_millis(200))
//!     .build()?;
//!
//! let q: serde_json::Value = client.queue().list(None)?;
//! println!("{q:?}");
//! # Ok(())
//! # }
//! ```

// compile-time guard: enable at least one client kind.
#[cfg(not(any(feature = "async", feature = "blocking")))]
compile_error!("Enable at least one of: `async` (default) or `blocking`.");

// compile-time guard: choose exactly one TLS backend.
#[cfg(all(
    any(feature = "async", feature = "blocking"),
    not(any(feature = "rustls", feature = "native-tls"))
))]
compile_error!("Enable exactly one TLS backend: `rustls` (default) or `native-tls`.");

#[cfg(all(feature = "rustls", feature = "native-tls"))]
compile_error!("`rustls` and `native-tls` features are mutually exclusive.");

pub mod api;
mod auth;
mod client;
mod error;
mod request_hook;
pub mod types;

mod transport;
mod util;

#[cfg(feature = "blocking")]
pub use api::{
    BlockingComputersService, BlockingJobsService, BlockingPeopleService, BlockingQueueService,
    BlockingSystemService, BlockingUsersService, BlockingViewsService,
};
#[cfg(feature = "async")]
pub use api::{
    ComputersService, JobsService, PeopleService, QueueService, SystemService, UsersService,
    ViewsService,
};
pub use auth::Auth;
pub use error::{BodySnippetConfig, Error, ErrorKind, HttpError, Result, TransportErrorKind};
pub use request_hook::{RequestHook, RequestHookContext};
pub use transport::middleware::RetryConfig;
pub use types::*;

#[cfg(feature = "blocking")]
pub use client::{BlockingClient, BlockingClientBuilder};
#[cfg(feature = "async")]
pub use client::{Client, ClientBuilder};

/// Escape hatch for advanced users.
///
/// This module is behind the `unstable-raw` feature and is not SemVer-stable.
#[cfg(feature = "unstable-raw")]
pub mod raw;
