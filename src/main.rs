mod service;
mod cri;

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing::{info, error};
use service::containerd::Server;

// 定义生成的protobuf代码模块
pub mod api {
    pub mod runtime {
        pub mod v1 {
            // 包含tonic生成的gRPC客户端和服务器
            tonic::include_proto!("runtime.v1");
            
            // 重新导出一些常用的类型
            pub use self::{
                runtime_service_client::RuntimeServiceClient,
                image_service_client::ImageServiceClient,
            };
        }
    }
}

const DEFAULT_CONTAINERD_ENDPOINT: &str = "unix:///run/containerd/containerd.sock";

// 替换 #[tokio::main] 宏
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main());
    Ok(())
}

async fn async_main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    info!("Starting MCP containerd server");
    
    let service = Server::new("unix:///run/containerd/containerd.sock".to_string()).serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;
    service.waiting().await?;
    Ok(())
}
