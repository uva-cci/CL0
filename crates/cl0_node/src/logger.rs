// use std::fs;
// use std::path::Path;
// use tracing_subscriber::{EnvFilter, fmt};

// pub fn setup_node_logger(node_id: &str) {
//     let log_dir = Path::new("logs");
//     fs::create_dir_all(log_dir).unwrap();

//     let file_path = log_dir.join(format!("{}.log", node_id));
//     let file = std::fs::File::create(file_path).expect("failed to create log file");

//     let subscriber = fmt::Subscriber::builder()
//         .with_writer(std::sync::Mutex::new(file)) // Thread-safe writer
//         .with_max_level(tracing::Level::INFO)
//         .with_target(false)
//         .with_env_filter(EnvFilter::from_default_env())
//         .finish();

//     // Only set the global subscriber once
//     let _ = tracing::subscriber::set_global_default(subscriber);
// }
