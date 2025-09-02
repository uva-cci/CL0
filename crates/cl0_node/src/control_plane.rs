use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use tonic::{Request, Response, Status};

use crate::generated;
use crate::generated::common::RuleResult;
use crate::generated::control_plane::{
    Ack, ControlMessage, HeartbeatAck, NodeAck, NodeHeartbeat, NodeRegistration,
    RuleExecutionRequest,
    control_plane_server::{ControlPlane, ControlPlaneServer},
};

type NodeId = String;

#[derive(Debug, Default)]
pub struct ControlPlaneService {
    pub state: Arc<RwLock<ControlPlaneState>>,
}

#[derive(Debug, Default)]
pub struct ControlPlaneState {
    pub nodes: HashMap<NodeId, RegisteredNode>,
    pub pools: HashMap<String, Vec<NodeId>>,
}

#[derive(Debug)]
pub struct RegisteredNode {
    pub hostname: String,
    pub last_heartbeat: i64,
    pub version: String,
}

#[tonic::async_trait]
impl ControlPlane for ControlPlaneService {
    async fn register_node(
        &self,
        request: Request<NodeRegistration>,
    ) -> Result<Response<NodeAck>, Status> {
        let req = request.into_inner();
        let id = Uuid::new_v4().to_string();
        let mut state = self.state.write().await;

        
        let registered_node = RegisteredNode {
            hostname: req.hostname,
            version: req.version,
            last_heartbeat: chrono::Utc::now().timestamp(),
        };

        // Store the registered node
        state.nodes.insert(id.clone(), registered_node);

        // Assign the node to a pool
        if let Some(pool_name) = req.pool {
            state
                .pools
                .entry(pool_name)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
        else {
            state
                .pools
                .entry(format!("pool-{}", id))
                .or_insert_with(Vec::new)
                .push(id.clone());
        }


        println!("New node registered: {id}");

        Ok(Response::new(NodeAck {
            id: Some(generated::common::NodeId { id }),
        }))
    }

    async fn heartbeat(
        &self,
        request: Request<NodeHeartbeat>,
    ) -> Result<Response<HeartbeatAck>, Status> {
        let req = request.into_inner();
        let mut state = self.state.write().await;

        if let Some(id) = req.id {
            if let Some(node) = state.nodes.get_mut(&id.id) {
                node.last_heartbeat = chrono::Utc::now().timestamp();
                println!("Heartbeat received from: {}", id.id);
                return Ok(Response::new(HeartbeatAck {}));
            }
        }

        Err(Status::not_found("Node not found"))
    }

    async fn forward_message(
        &self,
        request: Request<ControlMessage>,
    ) -> Result<Response<Ack>, Status> {
        let msg = request.into_inner();
        println!(
            "Received message from node {}: {}",
            msg.sender
                .unwrap_or(generated::common::NodeId {
                    id: "unknown".into()
                })
                .id,
            msg.message
        );

        // TODO: handle message distribution

        Ok(Response::new(Ack {}))
    }

    async fn request_rule_execution(
        &self,
        request: Request<RuleExecutionRequest>,
    ) -> Result<Response<RuleResult>, Status> {
        let req = request.into_inner();
        println!("Received rule execution request for node {:?}", req.target);

        // TODO: forward rule to appropriate node and wait for response
        Ok(Response::new(RuleResult {
            success: true,
            output: "Rule execution not yet implemented".into(),
        }))
    }
}

pub fn new_service_instance() -> (
    ControlPlaneServer<ControlPlaneService>,
    Arc<RwLock<ControlPlaneState>>,
) {
    let service: ControlPlaneService = ControlPlaneService::default();
    let state: Arc<RwLock<ControlPlaneState>> = service.state.clone();
    
    (ControlPlaneServer::new(service), state)
}
