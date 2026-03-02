/// Trust root selection for HTTPS requests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TlsRootStore {
    /// Use the transport backend default trust roots.
    #[default]
    BackendDefault,
    /// Force WebPKI roots.
    WebPki,
    /// Use platform/system trust verification.
    System,
}

impl TlsRootStore {
    #[cfg(any(feature = "async", feature = "blocking"))]
    pub(crate) const fn into_reqx(self) -> reqx::TlsRootStore {
        match self {
            Self::BackendDefault => reqx::TlsRootStore::BackendDefault,
            Self::WebPki => reqx::TlsRootStore::WebPki,
            Self::System => reqx::TlsRootStore::System,
        }
    }
}
