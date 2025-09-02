use tonic::transport::Channel;
use tonic::Request;
use tokio::time::{sleep, Duration};
use chrono::Utc;

use crate::generated::control_plane::{
    control_plane_client::ControlPlaneClient,
    NodeRegistration, NodeHeartbeat, ControlMessage,
};
use crate::generated::common::{NodeId, Rule};

#[derive(Debug)]
pub struct NodeClient {
    pub client: ControlPlaneClient<Channel>,
    pub node_id: Option<NodeId>,
    pub pool: Option<String>,
    pub hostname: String,
    pub version: String,
}

impl NodeClient {
    pub async fn new(endpoint: &str, version: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(endpoint.to_string())?
            .connect()
            .await?;

        let hostname = gethostname::gethostname()
            .to_str()
            .unwrap_or("unknown")
            .to_string();

        Ok(NodeClient {
            client: ControlPlaneClient::new(channel),
            node_id: None,
            pool: None,
            hostname,
            version: version.to_string(),
        })
    }
    pub async fn new_with_pool(endpoint: &str, version: &str, pool: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let node = NodeClient::new(endpoint, version).await?;
        Ok(NodeClient {
            client: node.client,
            node_id: node.node_id,
            pool: Some(pool.to_string()),
            hostname: node.hostname,
            version: node.version,
        })
    }

    pub async fn register(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let registration = NodeRegistration {
            hostname: self.hostname.clone(),
            version: self.version.clone(),
            pool: self.pool.clone(),
        };

        let response = self.client.register_node(Request::new(registration)).await?;
        self.node_id = response.into_inner().id;

        println!("✅ Registered as node {:?}", self.node_id);
        Ok(())
    }

    pub async fn send_heartbeat_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if let Some(ref node_id) = self.node_id {
                let heartbeat = NodeHeartbeat {
                    id: Some(node_id.clone()),
                    timestamp: Utc::now().timestamp(),
                };

                match self.client.heartbeat(Request::new(heartbeat)).await {
                    Ok(_) => println!("❤️ Sent heartbeat"),
                    Err(e) => eprintln!("💥 Failed to send heartbeat: {:?}", e),
                }
            }

            sleep(Duration::from_secs(5)).await;
        }
    }

    pub async fn forward_message(
        &mut self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref node_id) = self.node_id {
            let msg = ControlMessage {
                sender: Some(node_id.clone()),
                message: message.to_string(),
            };

            self.client.forward_message(Request::new(msg)).await?;
            println!("📨 Message forwarded");
        }

        Ok(())
    }

    pub async fn request_rule_execution(
        &mut self,
        target_node: NodeId,
        rule: Rule,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::generated::control_plane::RuleExecutionRequest;

        let req = RuleExecutionRequest {
            target: Some(target_node),
            rule: Some(rule),
        };

        let res = self.client.request_rule_execution(Request::new(req)).await?;
        println!("✅ Rule result: {:?}", res.into_inner());

        Ok(())
    }
}
