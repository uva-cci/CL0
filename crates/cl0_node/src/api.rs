use std::{error::Error, future::Future, sync::Arc};
use tokio::{
    sync::{mpsc, oneshot},
    task,
};

/// Wrapper around results returned from the API handlers.
pub type ApiResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// A generic route representing a message/command sink with an optional reply channel.
///
/// Provides two invocation styles:
///  - `notify`: fire-and-forget. The caller does not await the handler's completion.
///  - `call`: RPC-style. The caller waits for the handler result.
#[derive(Debug)]
pub struct ApiRoute<Req, Res> {
    tx: mpsc::UnboundedSender<(Req, Option<oneshot::Sender<ApiResult<Res>>>)>,
}

impl<Req, Res> Clone for ApiRoute<Req, Res> {
    fn clone(&self) -> Self {
        ApiRoute {
            tx: self.tx.clone(),
        }
    }
}

impl<Req, Res> ApiRoute<Req, Res>
where
    Req: Send + Sync + 'static,
    Res: Send + 'static,
{
    /// Constructs a new route backed by a background dispatcher. Each incoming request is
    /// handled by the provided `handler` asynchronously; the handler may return a result.
    pub fn new<H, F>(handler: H) -> Self
    where
        H: Fn(Req) -> F + Send + Sync + 'static,
        F: Future<Output = ApiResult<Res>> + Send + 'static,
    {
        let (tx, mut rx) =
            mpsc::unbounded_channel::<(Req, Option<oneshot::Sender<ApiResult<Res>>>)>();
        let handler = Arc::new(handler);

        // Main dispatcher loop: consume incoming requests and spawn their handlers.
        task::spawn(async move {
            while let Some((req, maybe_ack)) = rx.recv().await {
                let handler = handler.clone();
                task::spawn(async move {
                    let result: ApiResult<Res> = handler(req).await;

                    // If the caller asked for a reply, send it back.
                    if let Some(ack_tx) = maybe_ack {
                        let _ = ack_tx.send(result);
                    }
                });
            }
        });

        ApiRoute { tx }
    }

    /// Fire-and-forget invocation: no result is awaited.
    pub fn notify(&self, req: Req) {
        let _ = self.tx.send((req, None));
    }

    /// RPC-style invocation: waits for the handler to complete and returns its result.
    pub async fn call(&self, req: Req) -> ApiResult<Res> {
        let (ack_tx, ack_rx) = oneshot::channel::<ApiResult<Res>>();
        self.tx
            .send((req, Some(ack_tx)))
            .map_err(|e| Box::<dyn Error + Send + Sync>::from(e))?;

        ack_rx
            .await
            .map_err(|e| Box::<dyn Error + Send + Sync>::from(e))?
    }
}
