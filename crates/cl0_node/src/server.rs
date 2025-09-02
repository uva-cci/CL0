use dashmap::DashMap;
use futures::{Stream, StreamExt};
use std::{net::SocketAddr, pin::Pin, sync::Arc};
use tokio::sync::{RwLock, broadcast};
use tokio_stream::wrappers::{BroadcastStream, ReceiverStream};
use tonic::{Request, Response, Status};
use tonic_web::GrpcWebLayer;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

// ===== Generated types =====
use crate::generated::web::{
    Ack,
    Empty,
    HistoryChunk,
    Input,
    Join,
    NodeDescriptor,
    NodePoolDescriptor,
    Output,
    Presence,
    PresenceEvent,
    PresenceSnapshot,
    PresenceUpdate,
    Scope,
    ServerEvent,
    ServerNotice,
    StatusSnapshot,
    SystemTree,
    // services
    control_plane_service_server::{ControlPlaneService, ControlPlaneServiceServer},
    presence_event,
    presence_service_server::{PresenceService, PresenceServiceServer},
    presence_update,
    repl_service_server::{ReplService, ReplServiceServer},
    scope,
    server_event,
    status_service_server::{StatusService, StatusServiceServer},
};

// =========================
// Core in-memory cluster
// =========================

/// Minimal REPL session for any scope (control plane, pool, node).
#[derive(Debug)]
struct ScopeSession {
    history: RwLock<Vec<Output>>,
    tx: broadcast::Sender<ServerEvent>,
}

impl ScopeSession {
    fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1024);
        Self {
            history: RwLock::new(Vec::new()),
            tx,
        }
    }
}

/// Node session = REPL + node status
#[derive(Debug)]
struct NodeSession {
    repl: Arc<ScopeSession>,
    status: RwLock<StatusSnapshot>,
}

impl NodeSession {
    fn new(node_id: &str) -> Self {
        Self {
            repl: Arc::new(ScopeSession::new()),
            status: RwLock::new(StatusSnapshot {
                scope: Some(Scope {
                    kind: scope::Kind::Node as i32,
                    id: node_id.to_string(),
                }),
                rules: Vec::new(),
                vars: Vec::new(),
            }),
        }
    }
}

/// Pool session = REPL + nodes map
#[derive(Debug)]
struct PoolSession {
    repl: Arc<ScopeSession>,
    nodes: DashMap<String, Arc<NodeSession>>,
    name: String,
}

impl PoolSession {
    fn new(id: &str, name: &str) -> Self {
        Self {
            repl: Arc::new(ScopeSession::new()),
            nodes: DashMap::new(),
            name: name.to_string(),
        }
    }
}

/// Entire cluster: control plane + pools (+ node index) + tree broadcast
#[derive(Debug)]
struct Cluster {
    control_plane_id: String,
    control: Arc<ScopeSession>,
    pools: DashMap<String, Arc<PoolSession>>,
    // For O(1) node lookup by id (assumes global uniqueness)
    nodes_index: DashMap<String, Arc<NodeSession>>,
    // System tree broadcaster
    tree_tx: broadcast::Sender<SystemTree>,
}

impl Cluster {
    fn new(control_plane_id: impl Into<String>) -> Self {
        let (tree_tx, _rx) = broadcast::channel(64);
        Self {
            control_plane_id: control_plane_id.into(),
            control: Arc::new(ScopeSession::new()),
            pools: DashMap::new(),
            nodes_index: DashMap::new(),
            tree_tx,
        }
    }

    fn ensure_pool(&self, pool_id: &str) -> Arc<PoolSession> {
        if let Some(p) = self.pools.get(pool_id) {
            return Arc::clone(&*p);
        }
        let pool = Arc::new(PoolSession::new(pool_id, pool_id));
        let entry = self
            .pools
            .entry(pool_id.to_string())
            .or_insert_with(|| Arc::clone(&pool));
        Arc::clone(&*entry)
    }

    fn ensure_node(&self, pool_id: &str, node_id: &str) -> Arc<NodeSession> {
        if let Some(n) = self.nodes_index.get(node_id) {
            return Arc::clone(&*n);
        }
        let pool = self.ensure_pool(pool_id);
        let node = Arc::new(NodeSession::new(node_id));
        pool.nodes.insert(node_id.to_string(), Arc::clone(&node));
        self.nodes_index
            .insert(node_id.to_string(), Arc::clone(&node));
        self.broadcast_tree(); // topology changed
        node
    }

    /// If you only know a node_id (global-unique), fetch/create under a default pool.
    fn ensure_node_global(&self, node_id: &str) -> Arc<NodeSession> {
        if let Some(n) = self.nodes_index.get(node_id) {
            return Arc::clone(&*n);
        }
        // Default grouping (adjust if you have a real pool mapping)
        self.ensure_node("default", node_id)
    }

    /// Convert current topology to a SystemTree snapshot.
    fn system_tree_snapshot(&self) -> SystemTree {
        let mut pools: Vec<NodePoolDescriptor> = Vec::new();
        for p in self.pools.iter() {
            let mut nds = Vec::new();
            for n in p.nodes.iter() {
                nds.push(NodeDescriptor {
                    id: n.key().clone(),
                    name: n.key().clone(),
                });
            }
            pools.push(NodePoolDescriptor {
                id: p.key().clone(),
                name: p.value().name.clone(),
                nodes: nds,
            });
        }
        SystemTree {
            control_plane_id: self.control_plane_id.clone(),
            node_pools: pools,
        }
    }

    fn broadcast_tree(&self) {
        let _ = self.tree_tx.send(self.system_tree_snapshot());
    }

    /// Get a REPL session for any scope.
    fn get_scope_session(&self, scope: &Scope) -> Result<Arc<ScopeSession>, Status> {
        match scope::Kind::try_from(scope.kind).unwrap_or(scope::Kind::Unspecified) {
            scope::Kind::ControlPlane => Ok(Arc::clone(&self.control)),
            scope::Kind::NodePool => Ok(self.ensure_pool(&scope.id).repl.clone()),
            scope::Kind::Node => Ok(self.ensure_node_global(&scope.id).repl.clone()),
            scope::Kind::Unspecified => Err(Status::invalid_argument("scope.kind unspecified")),
        }
    }

    /// Get a NodeSession (node-level)
    fn get_node_session(&self, scope: &Scope) -> Result<Arc<NodeSession>, Status> {
        match scope::Kind::try_from(scope.kind).unwrap_or(scope::Kind::Unspecified) {
            scope::Kind::Node => Ok(self.ensure_node_global(&scope.id)),
            _ => Err(Status::invalid_argument("GetStatus requires NODE scope")),
        }
    }

    /// Public helpers if your control-plane integration wants to upsert pools/nodes:
    #[allow(dead_code)]
    fn upsert_pool(&self, pool_id: &str, name: &str) {
        let _ = self
            .pools
            .entry(pool_id.to_string())
            .or_insert_with(|| Arc::new(PoolSession::new(pool_id, name)));
        self.broadcast_tree();
    }

    #[allow(dead_code)]
    fn upsert_node(&self, pool_id: &str, node_id: &str, _name: &str) {
        let _ = self.ensure_node(pool_id, node_id);
    }
}

// ==========================
// Presence hub (global)
// ==========================

#[derive(Debug)]
struct PresenceHub {
    users: DashMap<String, Presence>, // user_id -> Presence
    tx: broadcast::Sender<PresenceEvent>,
}

impl Default for PresenceHub {
    fn default() -> Self {
        let (tx, _rx) = broadcast::channel(512); // capacity chosen by you
        PresenceHub {
            users: DashMap::new(),
            tx,
        }
    }
}

impl PresenceHub {
    fn new() -> Self {
        let (tx, _rx) = broadcast::channel(256);
        Self {
            users: DashMap::new(),
            tx,
        }
    }

    fn users_in_scope(&self, target: &Scope) -> Vec<Presence> {
        self.users
            .iter()
            .filter(|e| e.value().scope.as_ref() == Some(target))
            .map(|e| e.value().clone())
            .collect()
    }

    fn snapshot(&self) -> PresenceSnapshot {
        let mut users = Vec::new();
        for u in self.users.iter() {
            users.push(u.value().clone());
        }
        PresenceSnapshot { users }
    }

    fn join(&self, p: Presence) {
        let user_id = p.user_id.clone();
        match self.users.insert(user_id.clone(), p.clone()) {
            None => {
                // JOINED
                let _ = self.tx.send(PresenceEvent {
                    kind: Some(presence_event::Kind::Update(PresenceUpdate {
                        kind: presence_update::Kind::Joined as i32,
                        user: Some(p),
                    })),
                });
            }
            Some(prev) => {
                // MOVED if scope changed
                if prev.scope != p.scope {
                    let _ = self.tx.send(PresenceEvent {
                        kind: Some(presence_event::Kind::Update(PresenceUpdate {
                            kind: presence_update::Kind::Moved as i32,
                            user: Some(p),
                        })),
                    });
                } // else: same scope, no-op
            }
        }
    }

    fn leave(&self, p: Presence) {
        let user_id = p.user_id.clone();
        if self.users.remove(&user_id).is_some() {
            let _ = self.tx.send(PresenceEvent {
                kind: Some(presence_event::Kind::Update(PresenceUpdate {
                    kind: presence_update::Kind::Left as i32,
                    user: Some(p),
                })),
            });
        }
    }
}

// ==========================
// Services
// ==========================

#[derive(Clone)]
struct ReplSvc {
    cluster: Arc<Cluster>,
}

#[tonic::async_trait]
impl ReplService for ReplSvc {
    type SubscribeStream = Pin<Box<dyn Stream<Item = Result<ServerEvent, Status>> + Send>>;

    async fn subscribe(
        &self,
        request: Request<Join>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let Join {
            user_id,
            scope: maybe_scope,
            since_id,
        } = request.into_inner();
        let scope = maybe_scope.ok_or_else(|| Status::invalid_argument("Join.scope required"))?;
        let session = self.cluster.get_scope_session(&scope)?;

        let (out_tx, out_rx) = tokio::sync::mpsc::channel::<Result<ServerEvent, Status>>(256);

        // Pump: history -> join notice -> forward broadcast -> leave notice
        tokio::spawn({
            let session = Arc::clone(&session);
            let scope_clone = scope.clone();
            let user_id_clone = user_id.clone();
            async move {
                // (a) history
                let items = {
                    let g = session.history.read().await;
                    if since_id.is_empty() {
                        g.clone()
                    } else {
                        // find first index with id > since_id
                        let idx = g.iter().position(|o| o.id >= since_id).unwrap_or(g.len());
                        g[idx..].to_vec()
                    }
                };
                let chunk = ServerEvent {
                    kind: Some(server_event::Kind::History(HistoryChunk {
                        scope: Some(scope_clone.clone()),
                        items,
                        done: true,
                    })),
                };
                if out_tx.send(Ok(chunk)).await.is_err() {
                    return;
                }

                // (b) join notice
                let _ = session.tx.send(ServerEvent {
                    kind: Some(server_event::Kind::Notice(ServerNotice {
                        scope: Some(scope_clone.clone()),
                        text: format!("{user_id_clone} joined"),
                    })),
                });

                // (c) forward live
                let mut live = BroadcastStream::new(session.tx.subscribe());
                while let Some(item) = live.next().await {
                    match item {
                        Ok(evt) => {
                            if out_tx.send(Ok(evt)).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(
                            _n,
                        )) => {
                            // Optional: send a resync chunk (latest N history)
                            let snapshot = {
                                let g = session.history.read().await;
                                // last up to 200 items, for example
                                let take = g.len().saturating_sub(200);
                                g.iter().skip(take).cloned().collect::<Vec<_>>()
                            };
                            let _ = out_tx
                                .send(Ok(ServerEvent {
                                    kind: Some(server_event::Kind::History(HistoryChunk {
                                        scope: Some(scope.clone()),
                                        items: snapshot,
                                        done: true,
                                    })),
                                }))
                                .await;
                        }
                    }
                }

                // (d) leave notice
                let _ = session.tx.send(ServerEvent {
                    kind: Some(server_event::Kind::Notice(ServerNotice {
                        scope: Some(scope_clone.clone()),
                        text: format!("{user_id_clone} left"),
                    })),
                });
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(out_rx))))
    }

    async fn send_command(&self, request: Request<Input>) -> Result<Response<Ack>, Status> {
        let Input {
            user_id,
            scope: maybe_scope,
            code,
        } = request.into_inner();
        let scope = maybe_scope.ok_or_else(|| Status::invalid_argument("Input.scope required"))?;
        let session = self.cluster.get_scope_session(&scope)?;

        // --- Execute your rule/REPL here ---
        let result_text = format!(">> {}\n{}", code, "[result placeholder]");

        let out = Output {
            id: uuid::Uuid::now_v7().to_string(),
            scope: Some(scope.clone()),
            user_id: user_id.clone(),
            stdout: result_text,
            unix_ts: chrono::Utc::now().timestamp(),
        };

        {
            let mut hist = session.history.write().await;
            hist.push(out.clone());
        }

        // fan-out
        let _ = session.tx.send(ServerEvent {
            kind: Some(server_event::Kind::Output(out.clone())),
        });

        Ok(Response::new(Ack {
            input_echo: code,
            output_id: out.id,
        }))
    }
}

#[derive(Clone)]
struct StatusSvc {
    cluster: Arc<Cluster>,
}

#[tonic::async_trait]
impl StatusService for StatusSvc {
    async fn get_status(
        &self,
        request: Request<Scope>,
    ) -> Result<Response<StatusSnapshot>, Status> {
        let scope = request.into_inner();
        let node = self.cluster.get_node_session(&scope)?;
        let snap = node.status.read().await.clone();
        Ok(Response::new(snap))
    }
}

#[derive(Clone)]
struct TreeSvc {
    cluster: Arc<Cluster>,
}

#[tonic::async_trait]
impl ControlPlaneService for TreeSvc {
    type SubscribeTreeStream = Pin<Box<dyn Stream<Item = Result<SystemTree, Status>> + Send>>;

    async fn subscribe_tree(
        &self,
        _request: Request<Scope>,
    ) -> Result<Response<Self::SubscribeTreeStream>, Status> {
        let (out_tx, out_rx) = tokio::sync::mpsc::channel::<Result<SystemTree, Status>>(64);

        // send snapshot immediately
        let snapshot = self.cluster.system_tree_snapshot();
        if out_tx.send(Ok(snapshot)).await.is_err() {
            return Ok(Response::new(Box::pin(ReceiverStream::new(out_rx))));
        }

        // forward updates
        tokio::spawn({
            let mut rx = BroadcastStream::new(self.cluster.tree_tx.subscribe());
            async move {
                while let Some(Ok(tree)) = rx.next().await {
                    if out_tx.send(Ok(tree)).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(out_rx))))
    }
}

#[derive(Clone)]
struct PresenceSvc {
    hub: Arc<PresenceHub>,
}

#[tonic::async_trait]
impl PresenceService for PresenceSvc {
    type SubscribeStream = Pin<Box<dyn Stream<Item = Result<PresenceEvent, Status>> + Send>>;

    async fn subscribe(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let (out_tx, out_rx) = tokio::sync::mpsc::channel::<Result<PresenceEvent, Status>>(256);

        // (1) send snapshot
        let snap = self.hub.snapshot();
        if out_tx
            .send(Ok(PresenceEvent {
                kind: Some(presence_event::Kind::Snapshot(snap)),
            }))
            .await
            .is_err()
        {
            return Ok(Response::new(Box::pin(ReceiverStream::new(out_rx))));
        }

        // (2) forward updates
        tokio::spawn({
            let mut rx = BroadcastStream::new(self.hub.tx.subscribe());
            async move {
                while let Some(Ok(evt)) = rx.next().await {
                    if out_tx.send(Ok(evt)).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(out_rx))))
    }

    async fn join(&self, request: Request<Presence>) -> Result<Response<Empty>, Status> {
        let p = request.into_inner();
        if p.user_id.is_empty() || p.scope.is_none() {
            return Err(Status::invalid_argument(
                "Presence requires user_id and scope",
            ));
        }
        self.hub.join(p);
        Ok(Response::new(Empty {}))
    }

    async fn leave(&self, request: Request<Presence>) -> Result<Response<Empty>, Status> {
        let p = request.into_inner();
        if p.user_id.is_empty() || p.scope.is_none() {
            return Err(Status::invalid_argument(
                "Presence requires user_id and scope",
            ));
        }
        self.hub.leave(p);
        Ok(Response::new(Empty {}))
    }
}

// ==========================
// Server bootstrap
// ==========================

pub async fn serve(socket_address: Option<SocketAddr>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = socket_address.unwrap_or(([127, 0, 0, 1], 50051).into());

    let cluster = Arc::new(Cluster::new("cp-1"));
    cluster.upsert_pool("default", "default");
    cluster.upsert_node("default", "node-1", "node-1");

    let presence = Arc::new(PresenceHub::new());

    let repl = ReplSvc {
        cluster: Arc::clone(&cluster),
    };
    let status = StatusSvc {
        cluster: Arc::clone(&cluster),
    };
    let tree = TreeSvc {
        cluster: Arc::clone(&cluster),
    };
    let pres = PresenceSvc {
        hub: Arc::clone(&presence),
    };

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    tonic::transport::Server::builder()
        .accept_http1(true)
        .layer(ServiceBuilder::new().layer(cors).layer(GrpcWebLayer::new()))
        .add_service(ReplServiceServer::new(repl))
        .add_service(StatusServiceServer::new(status))
        .add_service(ControlPlaneServiceServer::new(tree))
        .add_service(PresenceServiceServer::new(pres))
        .serve(addr)
        .await?;

    Ok(())
}
