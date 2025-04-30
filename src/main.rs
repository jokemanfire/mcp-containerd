mod cri;
mod service;

use anyhow::Result;
use clap::Parser;
use rmcp::transport::sse_server::SseServer;
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

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:3000";
const DEFAULT_CONTAINERD_ENDPOINT: &str = "unix:///run/containerd/containerd.sock";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Transport type: stdio or sse
    #[arg(short, long, default_value = "stdio")]
    transport: String,

    /// Bind port for SSE transport
    #[arg(short, long, default_value = DEFAULT_BIND_ADDRESS)]
    address: String,

    /// Containerd endpoint
    #[arg(short, long, default_value = DEFAULT_CONTAINERD_ENDPOINT)]
    endpoint: String,
}

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

    let args = Args::parse();
    let container_server = Server::new(args.endpoint.clone());
    container_server.connect().await.unwrap();
    match args.transport.as_str() {
        "stdio" => {
            tracing::info!("Using stdio transport");
            let service = container_server
                .serve((tokio::io::stdin(), tokio::io::stdout()))
                .await
                .inspect_err(|e| {
                    tracing::error!("serving error: {:?}", e);
                })?;
            service.waiting().await?;
        }
        "sse" => {
            tracing::info!("Using SSE transport on {}", args.address);
            let ct = SseServer::serve(args.address.parse()?)
                .await?
                .with_service(move || container_server.clone());
            tokio::signal::ctrl_c().await?;
            ct.cancel();
        }
        _ => {
            tracing::error!("Invalid transport type: {}", args.transport);
            return Err(anyhow::anyhow!(
                "Invalid transport type: {}",
                args.transport
            ));
        }
    }

    Ok(())
}
