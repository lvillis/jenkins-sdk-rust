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
//! let q = client.queue().list(None).await?;
//! println!("queue items={}", q.items.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Quick start (blocking)
//! Enable one blocking transport feature (for example `blocking-rustls-ring`):
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
//! let q = client.queue().list(None)?;
//! println!("queue items={}", q.items.len());
//! # Ok(())
//! # }
//! ```

// compile-time guard: enable at least one client kind.
#[cfg(not(any(feature = "async", feature = "blocking")))]
compile_error!("Enable at least one of: `async` (default) or `blocking`.");

// compile-time guard: async mode requires exactly one async TLS backend.
#[cfg(all(
    feature = "async",
    not(any(
        feature = "async-rustls-ring",
        feature = "async-rustls-aws-lc-rs",
        feature = "async-native-tls"
    ))
))]
compile_error!(
    "Enable one async TLS backend: `async-rustls-ring` (default), `async-rustls-aws-lc-rs`, or `async-native-tls`."
);

#[cfg(all(
    feature = "async",
    any(
        all(feature = "async-rustls-ring", feature = "async-rustls-aws-lc-rs"),
        all(feature = "async-rustls-ring", feature = "async-native-tls"),
        all(feature = "async-rustls-aws-lc-rs", feature = "async-native-tls")
    )
))]
compile_error!(
    "Async mode accepts exactly one TLS backend: choose only one of `async-rustls-ring`, `async-rustls-aws-lc-rs`, or `async-native-tls`."
);

// compile-time guard: blocking mode requires exactly one blocking TLS backend.
#[cfg(all(
    feature = "blocking",
    not(any(
        feature = "blocking-rustls-ring",
        feature = "blocking-rustls-aws-lc-rs",
        feature = "blocking-native-tls"
    ))
))]
compile_error!(
    "Enable one blocking TLS backend: `blocking-rustls-ring`, `blocking-rustls-aws-lc-rs`, or `blocking-native-tls`."
);

#[cfg(all(
    feature = "blocking",
    any(
        all(
            feature = "blocking-rustls-ring",
            feature = "blocking-rustls-aws-lc-rs"
        ),
        all(feature = "blocking-rustls-ring", feature = "blocking-native-tls"),
        all(feature = "blocking-rustls-aws-lc-rs", feature = "blocking-native-tls")
    )
))]
compile_error!(
    "Blocking mode accepts exactly one TLS backend: choose only one of `blocking-rustls-ring`, `blocking-rustls-aws-lc-rs`, or `blocking-native-tls`."
);

pub mod api;
mod auth;
mod client;
mod error;
mod request_hook;
mod tls;
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
pub use tls::TlsRootStore;
pub use transport::middleware::RetryConfig;
pub use types::*;

#[cfg(feature = "blocking")]
pub use client::{BlockingClient, BlockingClientBuilder};
#[cfg(feature = "async")]
pub use client::{Client, ClientBuilder};
