//! Unstable escape hatch exposing internal request/transport pieces.
//!
//! This module is behind the `unstable-raw` feature and is not SemVer-stable.

pub use crate::transport::request::{Request, RequestBody, Response};

pub mod request {
    pub use crate::transport::request::{Request, RequestBody, Response};
}

pub mod transport {
    pub use crate::transport::{ResponseMeta, TransportBody, TransportRequest, TransportResponse};

    #[cfg(feature = "async")]
    pub use crate::transport::async_transport::*;
    #[cfg(feature = "blocking")]
    pub use crate::transport::blocking_transport::*;
}

pub mod middleware {
    pub use crate::transport::middleware::*;
}
