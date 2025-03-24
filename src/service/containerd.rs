use crate::api::runtime::v1::{
    RuntimeServiceClient, ImageServiceClient,
    VersionResponse, VersionRequest,
    ListPodSandboxRequest, ListPodSandboxResponse,
    ListContainersRequest, ListContainersResponse,
    ListImagesRequest, ListImagesResponse,
    ImageFsInfoRequest, ImageFsInfoResponse,
};
use anyhow::Result;
use futures::future::ok;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use rmcp::{
    Error as McpError, RoleServer, ServerHandler, const_string, model::*, schemars,
    service::RequestContext, tool,
};

#[derive(Clone)]
pub struct Server {
    endpoint: String,
    runtime_client: Arc<Mutex<Option<RuntimeServiceClient<tonic::transport::Channel>>>>,
    image_client: Arc<Mutex<Option<ImageServiceClient<tonic::transport::Channel>>>>,
}
#[tool(tool_box)]
impl Server {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            runtime_client: Arc::new(Mutex::new(None)),
            image_client: Arc::new(Mutex::new(None)),
        }
    }
    
    /// 连接到containerd服务
    pub async fn connect(&self) -> Result<()> {
        let channel = tonic::transport::Channel::from_shared(self.endpoint.clone())?
            .connect()
            .await?;
        
        // 初始化runtime客户端
        {
            let mut lock = self.runtime_client.lock().await;
            *lock = Some(RuntimeServiceClient::new(channel.clone()));
        }
        
        // 初始化image客户端
        {
            let mut lock = self.image_client.lock().await;
            *lock = Some(ImageServiceClient::new(channel));
        }
        
        Ok(())
    }
    
    #[tool(description = "Get version information")]
    pub async fn version(&self) -> Result<CallToolResult,McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let request = VersionRequest {
                version: "v1".to_string(),
            };
            let response = client.clone().version(request).await.unwrap();
            return Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&response.into_inner()).unwrap())]));
        }
        Ok(CallToolResult::success(vec![Content::text("")]))
    }
    
    #[tool(description = "List all pods")]
    pub async fn list_pods(&self) -> Result<CallToolResult,McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let request = ListPodSandboxRequest { filter:None };
            let response = client.clone().list_pod_sandbox(request).await.unwrap();
            return Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&response.into_inner()).unwrap())]));
        }
        Ok(CallToolResult::success(vec![Content::text("")]))
    }
    
    #[tool(description = "List all containers")]
    pub async fn list_containers(&self) -> Result<CallToolResult,McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let request = ListContainersRequest { filter:None };
            let response = client.clone().list_containers(request).await.unwrap();
            return Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&response.into_inner()).unwrap())]));
        }
        Ok(CallToolResult::success(vec![Content::text("")]))
    }
    
    #[tool(description = "List all images")]
    pub async fn list_images(&self) -> Result<CallToolResult,McpError> {
        let lock = self.image_client.lock().await;
        if let Some(client) = &*lock {
            let request = ListImagesRequest { filter:None };
            let response = client.clone().list_images(request).await.unwrap();
            return Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&response.into_inner()).unwrap())]));
        }
        Ok(CallToolResult::success(vec![Content::text("")]))
    }
    
    #[tool(description = "Get image file system information")]
    pub async fn image_fs_info(&self) -> Result<CallToolResult, McpError> {
        let lock = self.image_client.lock().await;
        if let Some(client) = &*lock {
            let request = ImageFsInfoRequest {};
            let response = client.clone().image_fs_info(request).await.unwrap();
            return Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&response.into_inner()).unwrap())]));
        }
        Ok(CallToolResult::success(vec![Content::text("")]))
    }
}




#[tool(tool_box)]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides a containerd tool that can list pods, containers, and images. Use 'version' to get the version information, 'list_pods' to list all pods, 'list_containers' to list all containers, and 'list_images' to list all images.".to_string()),
        }
    }

    async fn list_resources(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match uri.as_str() {
            "str:////Users/to/some/path/" => {
                let cwd = "/Users/to/some/path/";
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(cwd, uri)],
                })
            }
            _ => Err(McpError::resource_not_found(
                "resource_not_found",
                Some(json!({
                    "uri": uri
                })),
            )),
        }
    }

    async fn list_prompts(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![Prompt::new(
                "example_prompt",
                Some("This is an example prompt that takes one required agrument, message"),
                Some(vec![PromptArgument {
                    name: "message".to_string(),
                    description: Some("A message to put in the prompt".to_string()),
                    required: Some(true),
                }]),
            )],
        })
    }

    async fn get_prompt(
        &self,
        GetPromptRequestParam { name, arguments: _ }: GetPromptRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match name.as_str() {
            "example_prompt" => {
                let prompt = "This is an example prompt with your message here: '{message}'";
                Ok(GetPromptResult {
                    description: None,
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt),
                    }],
                })
            }
            _ => Err(McpError::invalid_params("prompt not found", None)),
        }
    }

    async fn list_resource_templates(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }
}