use cl0_node::server::serve;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    serve(None).await
}