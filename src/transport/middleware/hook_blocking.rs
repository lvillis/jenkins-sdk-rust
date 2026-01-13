use crate::transport::{
    TransportRequest, TransportResponse, blocking_transport::BlockingTransport,
};
use crate::{Error, RequestHook, RequestHookContext};

/// Blocking transport wrapper that executes a request hook before sending.
#[derive(Clone)]
pub struct HookBlocking<T> {
    inner: T,
    hook: RequestHook,
}

impl<T> HookBlocking<T> {
    pub fn new(inner: T, hook: RequestHook) -> Self {
        Self { inner, hook }
    }
}

impl<T: BlockingTransport> BlockingTransport for HookBlocking<T> {
    fn send(&self, mut req: TransportRequest) -> Result<TransportResponse, Error> {
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

        self.inner.send(req)
    }
}
