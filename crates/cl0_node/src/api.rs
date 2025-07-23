use std::{error::Error, future::Future, sync::Arc};
use tokio::{sync::{mpsc, oneshot}, task};


pub type ApiResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub struct ApiRoute<Req, Res> {
    tx: mpsc::UnboundedSender<(Req, Option<oneshot::Sender<ApiResult<Res>>>)>,
}

impl<Req, Res> Clone for ApiRoute<Req, Res> {
    fn clone(&self) -> Self {
        ApiRoute { tx: self.tx.clone() }
    }
}

impl<Req, Res> ApiRoute<Req, Res>
where
    Req: Send + Sync + 'static,
    Res: Send + 'static,
{
    /// `handler` now returns `ApiResult<Res>`.
    pub fn new<H, F>(handler: H) -> Self
    where
        H: Fn(Req) -> F + Send + Sync + 'static,
        F: Future<Output = ApiResult<Res>> + Send + 'static,
    {
        let (tx, mut rx) =
            mpsc::unbounded_channel::<(Req, Option<oneshot::Sender<ApiResult<Res>>>)>();
        let handler = Arc::new(handler);

        // dispatcher
        task::spawn(async move {
            while let Some((req, maybe_ack)) = rx.recv().await {
                let handler = handler.clone();
                task::spawn(async move {
                    // run the handler
                    let result: ApiResult<Res> = handler(req).await;

                    // if caller wanted the result, send it (moves `result`)
                    if let Some(ack_tx) = maybe_ack {
                        let _ = ack_tx.send(result);
                    }
                });
            }
        });

        ApiRoute { tx }
    }

    /// fire‑and‑forget; only channel‐send errors get lost
    pub fn notify(&self, req: Req) {
        let _ = self.tx.send((req, None));
    }

    /// RPC‑style: returns whatever your handler returned
    pub async fn call(&self, req: Req) -> ApiResult<Res> {
        let (ack_tx, ack_rx) = oneshot::channel::<ApiResult<Res>>();
        self.tx
            .send((req, Some(ack_tx)))
            .map_err(|e| Box::<dyn Error + Send + Sync>::from(e))?;

        ack_rx.await
            .map_err(|e| Box::<dyn Error + Send + Sync>::from(e))?
    }
}