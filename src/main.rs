mod cri;
mod service;

use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use service::containerd::Server;
use tracing_subscriber::{self, EnvFilter};

pub mod api {
    pub mod runtime {
        pub mod v1 {
            tonic::include_proto!("runtime.v1");

            pub use self::{
                image_service_client::ImageServiceClient,
                runtime_service_client::RuntimeServiceClient,
            };
        }
    }
}

const DEFAULT_CONTAINERD_ENDPOINT: &str = "unix:///run/containerd/containerd.sock";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main());
    Ok(())
}

async fn async_main() -> Result<()> {
    // init logger
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
    tracing::info!("Starting MCP server");
    let container_server = Server::new(DEFAULT_CONTAINERD_ENDPOINT.to_string());
    container_server
        .connect()
        .await
        .expect("connect containerd failed");
    let service = container_server
        .serve((tokio::io::stdin(), tokio::io::stdout()))
        .await
        .inspect_err(|e| {
            tracing::error!("serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
