use crate::transport::{TransportRequest, TransportResponse, async_transport::AsyncTransport};
use crate::{Error, RequestHook, RequestHookContext};
use async_trait::async_trait;

/// Async transport wrapper that executes a request hook before sending.
#[derive(Clone)]
pub struct HookAsync<T> {
    inner: T,
    hook: RequestHook,
}

impl<T> HookAsync<T> {
    pub fn new(inner: T, hook: RequestHook) -> Self {
        Self { inner, hook }
    }
}

#[async_trait]
impl<T: AsyncTransport> AsyncTransport for HookAsync<T> {
    async fn send(&self, mut req: TransportRequest) -> Result<TransportResponse, Error> {
        let body = req.body.as_ref();
        let body_bytes = body.map(|b| b.bytes.as_slice());
        let content_type = body.and_then(|b| b.content_type.as_ref());

        (self.hook)(RequestHookContext {
            method: &req.method,
            url: &req.url,
            headers: &mut req.headers,
            query: &req.query,
            form: &req.form,
            body: body_bytes,
            content_type,
        })?;

        self.inner.send(req).await
    }
}
