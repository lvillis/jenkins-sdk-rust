// compile-time guard: enable at least one client kind.
#[cfg(not(any(feature = "async-client", feature = "blocking-client")))]
compile_error!("Enable at least one of: `async-client` (default) or `blocking-client`.");

/// Jenkins-SDK – choose **async** *or* **blocking** at compile time.
pub mod core;
pub mod middleware;
pub mod transport;

#[cfg(feature = "async-client")]
pub mod client_async;
#[cfg(feature = "blocking-client")]
pub mod client_blocking;

#[cfg(feature = "async-client")]
pub use client_async::{JenkinsAsync, JenkinsAsyncBuilder};
#[cfg(feature = "blocking-client")]
pub use client_blocking::{JenkinsBlocking, JenkinsBlockingBuilder};
pub use core::*;
