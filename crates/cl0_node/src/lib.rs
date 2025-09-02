pub mod node;
pub mod event_handler;
pub mod control_plane;
mod logger;
pub mod utils;
pub mod types;
mod api;
pub mod visitor;
pub mod node_client;
pub mod server;

pub mod generated {
    pub mod common;
    pub mod node;
    pub mod control_plane;
    pub mod web; 
}

// pub mod common_proto {
//     tonic::include_proto!("common");
// }
// pub mod node_proto {
//     tonic::include_proto!("node");
// }
// pub mod control_plane_proto {
//     tonic::include_proto!("control_plane");
// }