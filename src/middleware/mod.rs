//! Re-exports for middleware layers.

#[cfg(feature = "async-client")]
pub mod retry_async;
#[cfg(feature = "blocking-client")]
pub mod retry_blocking;

#[cfg(feature = "async-client")]
pub mod crumb_async;
#[cfg(feature = "blocking-client")]
pub mod crumb_blocking;

/* public aliases */
#[cfg(feature = "async-client")]
pub use retry_async::RetryAsync as Retry;
#[cfg(all(feature = "blocking-client", not(feature = "async-client")))]
pub use retry_blocking::RetryBlocking as Retry;

#[cfg(feature = "async-client")]
pub use crumb_async::CrumbAsync as Crumb;
#[cfg(all(feature = "blocking-client", not(feature = "async-client")))]
pub use crumb_blocking::CrumbBlocking as Crumb;
