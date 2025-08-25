mod cri;
mod ctr;
mod service;
use anyhow::Result;
use clap::Parser;
use rmcp::transport::sse_server::SseServer;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::StreamableHttpService;
use rmcp::ServiceExt;
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
    /// Transport type: stdio or sse or http
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
    container_server
        .connect()
        .await
        .expect("Failed to connect to containerd, please check the endpoint");
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
        "http" => {
            tracing::info!("Using HTTP transport on {}", args.address);
            let service = StreamableHttpService::new(
                move || Ok(container_server.clone()),
                LocalSessionManager::default().into(),
                Default::default(),
            );

            let router = axum::Router::new().nest_service("/mcp", service);
            let tcp_listener = tokio::net::TcpListener::bind(DEFAULT_BIND_ADDRESS).await?;

            tracing::info!(
                "MCP HTTP server started at http://{}/mcp",
                DEFAULT_BIND_ADDRESS
            );
            tracing::info!("Press Ctrl+C to shutdown");

            let _ = axum::serve(tcp_listener, router)
                .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
                .await;
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
