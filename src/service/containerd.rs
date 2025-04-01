/*
 * Containerd Service Implementation
 *
 * Current Supported Tool Interfaces:
 * - version: Get the runtime version information
 * - list_pods: List all pod sandboxes
 * - list_containers: List all containers
 * - list_images: List all images
 * - image_fs_info: Get image filesystem information
 * - create_pod: Create a new pod sandbox
 * - remove_pod: Remove a pod sandbox
 * - create_container: Create a new container
 * - remove_container: Remove a container
 *
 * Future Planned Interfaces:
 * - stop_pod: Stop a running pod sandbox
 * - start_container: Start a created container
 * - stop_container: Stop a running container
 * - exec: Execute a command in a running container
 * - pull_image: Pull an image from registry
 * - remove_image: Remove an image
 * - container_stats: Get container statistics
 * - pod_stats: Get pod statistics
 * - container_logs: Get container logs
 */

use crate::api::runtime::v1::{
    ContainerConfig, CreateContainerRequest, CreateContainerResponse, ImageFsInfoRequest,
    ImageFsInfoResponse, ImageServiceClient, LinuxContainerConfig, LinuxPodSandboxConfig,
    ListContainersRequest, ListContainersResponse, ListImagesRequest, ListImagesResponse,
    ListPodSandboxRequest, ListPodSandboxResponse, Mount, PodSandboxConfig, RemoveContainerRequest,
    RemoveContainerResponse, RemovePodSandboxRequest, RemovePodSandboxResponse,
    RunPodSandboxRequest, RunPodSandboxResponse, RuntimeServiceClient, VersionRequest,
    VersionResponse,
};
use anyhow::Result;
use rmcp::{
    const_string, model::*, schemars, service::RequestContext, tool, Error as McpError, RoleServer,
    ServerHandler,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

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

    pub async fn connect(&self) -> Result<()> {
        let socket_path = self
            .endpoint
            .strip_prefix("unix://")
            .expect("endpoint must start with unix://")
            .to_string();

        let channel = tonic::transport::Endpoint::try_from("http://[::]:50051")?
            .connect_with_connector(tower::service_fn(move |_: tonic::transport::Uri| {
                let socket_path = socket_path.to_string();
                async move { tokio::net::UnixStream::connect(socket_path).await }
            }))
            .await?;

        {
            debug!("connect runtime client");
            let mut lock = self.runtime_client.lock().await;
            *lock = Some(RuntimeServiceClient::new(channel.clone()));
        }

        {
            debug!("connect image client");
            let mut lock = self.image_client.lock().await;
            *lock = Some(ImageServiceClient::new(channel));
        }

        Ok(())
    }

    #[tool(description = "Get version information")]
    pub async fn version(&self) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let request = VersionRequest {
                version: "v1".to_string(),
            };
            let response = client.clone().version(request).await.unwrap();
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string(&response.into_inner()).unwrap(),
            )]));
        }
        Ok(CallToolResult::success(vec![Content::text("")]))
    }

    #[tool(description = "List all pods")]
    pub async fn list_pods(&self) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let request = ListPodSandboxRequest { filter: None };
            let response = client.clone().list_pod_sandbox(request).await.unwrap();
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string(&response.into_inner()).unwrap(),
            )]));
        }
        Ok(CallToolResult::success(vec![Content::text("")]))
    }

    #[tool(description = "List all containers")]
    pub async fn list_containers(&self) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let request = ListContainersRequest { filter: None };
            let response = client.clone().list_containers(request).await.unwrap();
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string(&response.into_inner()).unwrap(),
            )]));
        }
        Ok(CallToolResult::success(vec![Content::text("")]))
    }

    #[tool(description = "List all images")]
    pub async fn list_images(&self) -> Result<CallToolResult, McpError> {
        let lock = self.image_client.lock().await;
        if let Some(client) = &*lock {
            let request = ListImagesRequest { filter: None };
            let response = client.clone().list_images(request).await.unwrap();
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string(&response.into_inner()).unwrap(),
            )]));
        }
        Ok(CallToolResult::success(vec![Content::text("")]))
    }

    #[tool(description = "Get image file system information")]
    pub async fn image_fs_info(&self) -> Result<CallToolResult, McpError> {
        let lock = self.image_client.lock().await;
        if let Some(client) = &*lock {
            let request = ImageFsInfoRequest {};
            let response = client.clone().image_fs_info(request).await.unwrap();
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string(&response.into_inner()).unwrap(),
            )]));
        }
        Ok(CallToolResult::success(vec![Content::text("")]))
    }

    #[tool(description = "Create a new pod sandbox")]
    pub async fn create_pod(
        &self,
        #[tool(param)]
        #[schemars(
            description = "{\"metadata\": {\"name\": \"my-pod\", \"namespace\": \"default\"}, \"hostname\": \"my-pod\"}"
        )]
        config: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // get pod config from json
            let pod_config: PodSandboxConfig = match serde_json::from_value(config) {
                Ok(config) => config,
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Invalid pod configuration: {}",
                        e
                    ))]));
                }
            };

            let request = RunPodSandboxRequest {
                config: Some(pod_config),
                runtime_handler: "".to_string(),
            };

            match client.clone().run_pod_sandbox(request).await {
                Ok(response) => {
                    let pod_id = response.into_inner().pod_sandbox_id;
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "{{\"pod_id\": \"{}\"}}",
                        pod_id
                    ))]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to create pod: {}",
                        e
                    ))]));
                }
            }
        }

        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    #[tool(description = "Remove a pod sandbox")]
    pub async fn remove_pod(
        &self,
        #[tool(param)]
        #[schemars(description = "{\"pod_id\": \"pod-12345\"}")]
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // get pod id from json
            let pod_id = match params.get("pod_id") {
                Some(id) => match id.as_str() {
                    Some(id_str) => id_str.to_string(),
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "pod_id must be a string",
                        )]));
                    }
                },
                None => {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Missing required parameter: pod_id",
                    )]));
                }
            };

            let request = RemovePodSandboxRequest {
                pod_sandbox_id: pod_id,
            };

            match client.clone().remove_pod_sandbox(request).await {
                Ok(_) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "{\"success\": true, \"message\": \"Pod removed successfully\"}",
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to remove pod: {}",
                        e
                    ))]));
                }
            }
        }

        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    #[tool(description = "Create a new container in a pod")]
    pub async fn create_container(
        &self,
        #[tool(param)]
        #[schemars(
            description = "{\"pod_id\": \"pod-12345\", \"config\": {\"metadata\": {\"name\": \"my-container\"}, \"image\": {\"image\": \"nginx:latest\"}, \"command\": [\"/bin/sh\"], \"args\": [\"-c\", \"while true; do echo hello; sleep 10; done\"]}}"
        )]
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // get pod id and container config from json
            let pod_id = match params.get("pod_id") {
                Some(id) => match id.as_str() {
                    Some(id_str) => id_str.to_string(),
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "pod_id must be a string",
                        )]));
                    }
                },
                None => {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Missing required parameter: pod_id",
                    )]));
                }
            };

            let container_config = match params.get("config") {
                Some(config) => match serde_json::from_value::<ContainerConfig>(config.clone()) {
                    Ok(config) => config,
                    Err(e) => {
                        return Ok(CallToolResult::error(vec![Content::text(format!(
                            "Invalid container configuration: {}",
                            e
                        ))]));
                    }
                },
                None => {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Missing required parameter: config",
                    )]));
                }
            };

            let request = CreateContainerRequest {
                pod_sandbox_id: pod_id,
                config: Some(container_config),
                sandbox_config: None,
            };

            match client.clone().create_container(request).await {
                Ok(response) => {
                    let container_id = response.into_inner().container_id;
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "{{\"container_id\": \"{}\"}}",
                        container_id
                    ))]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to create container: {}",
                        e
                    ))]));
                }
            }
        }

        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    #[tool(description = "Remove a container")]
    pub async fn remove_container(
        &self,
        #[tool(param)]
        #[schemars(description = "{\"container_id\": \"container-12345\"}")]
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // get container id from json
            let container_id = match params.get("container_id") {
                Some(id) => match id.as_str() {
                    Some(id_str) => id_str.to_string(),
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "container_id must be a string",
                        )]));
                    }
                },
                None => {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Missing required parameter: container_id",
                    )]));
                }
            };

            let request = RemoveContainerRequest { container_id };

            match client.clone().remove_container(request).await {
                Ok(_) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "{\"success\": true, \"message\": \"Container removed successfully\"}",
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to remove container: {}",
                        e
                    ))]));
                }
            }
        }

        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }
}
const_string!(Echo = "echo");
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
            instructions: Some("This server provides tools to interact with Containerd CRI. Currently supported tools: 'version' to get version information, 'list_pods' to list all pods, 'list_containers' to list all containers, 'list_images' to list all images, 'image_fs_info' to get image filesystem information, 'create_pod' to create a new pod, 'remove_pod' to remove a pod, 'create_container' to create a new container, and 'remove_container' to remove a container. Future updates will add more capabilities for container and pod management.".to_string()),
        }
    }

    async fn list_resources(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![],
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
