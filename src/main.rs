mod service;
mod cri;

use std::sync::Arc;
use anyhow::Result;
use tokio::io::{stdin, stdout};
use tracing::{info, error};
use service::containerd::{VersionService, ContainerdRuntimeService, ContainerdImageService};
use cri::runtime::RuntimeService;
use cri::image::ImageService;
use rmcp::handler::server::ServerHandler;

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

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    info!("Starting MCP containerd server");
    
    // 创建CRI客户端服务
    let runtime_service = Arc::new(RuntimeService::new());
    let image_service = Arc::new(ImageService::new());
    
    // 连接到containerd
    info!("Connecting to containerd at {}", DEFAULT_CONTAINERD_ENDPOINT);
    match runtime_service.connect(DEFAULT_CONTAINERD_ENDPOINT).await {
        Ok(_) => info!("Connected to containerd runtime service"),
        Err(e) => {
            error!("Failed to connect to containerd runtime service: {}", e);
            return Err(e);
        }
    }
    
    match image_service.connect(DEFAULT_CONTAINERD_ENDPOINT).await {
        Ok(_) => info!("Connected to containerd image service"),
        Err(e) => {
            error!("Failed to connect to containerd image service: {}", e);
            return Err(e);
        }
    }
    
    // 创建服务处理器
    let version_service = VersionService::new(runtime_service.clone());
    let runtime = ContainerdRuntimeService::new(runtime_service);
    let image = ContainerdImageService::new(image_service);
    
    // 创建处理器映射
    let mut handlers = std::collections::HashMap::new();
    handlers.insert("version".to_string(), Arc::new(version_service) as Arc<dyn ServerHandler>);
    handlers.insert("runtime".to_string(), Arc::new(runtime) as Arc<dyn ServerHandler>);
    handlers.insert("image".to_string(), Arc::new(image) as Arc<dyn ServerHandler>);
    
    // 启动服务器
    info!("Starting MCP server");
    let transport = (stdin(), stdout());
    let server = rmcp::service::serve_server(handlers, transport).await?;
    
    // 等待服务器关闭
    info!("MCP server running");
    let reason = server.waiting().await?;
    info!("MCP server stopped: {:?}", reason);
    
    Ok(())
}
