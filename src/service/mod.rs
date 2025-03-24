pub mod containerd;

use rmcp::handler::server::{RequestHandler, HandlerRegistry, ServerHandler, Root};
use anyhow::Result;
use std::sync::Arc;

// MCP服务器根节点
pub struct ContainerdService {
    // 这里可以添加服务状态
}

impl ContainerdService {
    pub fn new() -> Self {
        Self {}
    }
}

impl Root for ContainerdService {
    fn list_roots(&self) -> Vec<String> {
        vec![
            "version".to_string(),
            "runtime".to_string(),
            "image".to_string(),
        ]
    }
}

impl ServerHandler for ContainerdService {}

// 创建MCP服务处理器
pub fn create_handler() -> Arc<dyn RequestHandler> {
    let mut registry = HandlerRegistry::new();
    
    // 注册根服务
    let service = ContainerdService::new();
    registry.register_root(service);
    
    // 返回处理器
    Arc::new(registry)
} 