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
use futures::executor::block_on;
use rmcp::{
    const_string, model::*, schemars, service::RequestContext, tool, Error as McpError, RoleServer,
    ServerHandler,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;
use crate::api::runtime::v1::PodSandboxMetadata;
use uuid::Uuid;

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

    #[tool(description = "Get containerd logs")]
    pub async fn get_containerd_logs(
        &self,
        #[tool(param)]
        #[schemars(
            description = "The path to the log file, the default is /var/log/containerd/containerd.log"
        )]
        path: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let path = path.unwrap_or_default();
        // check if the file exists
        if !std::path::Path::new(&path).exists() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "File {} does not exist",
                path
            ))]));
        }
        // read the file
        let content = std::fs::read_to_string(path).unwrap();
        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    #[tool(description = "Get version information from containerd runtime")]
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
        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
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
        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
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
        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
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
        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
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
        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    #[tool(description = "Create a new pod sandbox")]
    pub async fn create_pod(
        &self,
        #[tool(param)]
        #[schemars(
            description = "
            the create pod config, the default is:
            {
                \"metadata\": {
                    \"name\": \"pod-name\",
                    \"uid\": \"pod-uid\",
                    \"namespace\": \"default\",
                    \"attempt\": 0
                },
                \"hostname\": \"pod-hostname\",
                \"log_directory\": \"/var/log/pods\",
                \"dns_config\": {
                    \"servers\": [\"8.8.8.8\"],
                    \"searches\": [\"example.com\"],
                    \"options\": [\"ndots:2\"]
                },
                \"port_mappings\": [
                    {
                        \"protocol\": \"TCP\",
                        \"container_port\": 80,
                        \"host_port\": 8080
                    }
                ],
                \"labels\": {
                    \"app\": \"nginx\"
                },
                \"annotations\": {
                    \"key\": \"value\"
                },
                \"linux\": {
                    \"cgroup_parent\": \"/kubepods\",
                    \"security_context\": {
                        \"namespace_options\": {
                            \"network\": \"POD\",
                            \"pid\": \"CONTAINER\"
                        }
                    }
                }
            }
            "
        )]
        config: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        debug!("create pod request: {:?}", config);
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // 创建默认配置
            let mut pod_config = PodSandboxConfig {
                metadata: Some(crate::api::runtime::v1::PodSandboxMetadata {
                    name: "default-pod".to_string(),
                    uid: uuid::Uuid::new_v4().to_string(),
                    namespace: "default".to_string(),
                    attempt: 0,
                }),
                hostname: "default-hostname".to_string(),
                log_directory: "/var/log/pods".to_string(),
                dns_config: None,
                port_mappings: vec![],
                labels: std::collections::HashMap::new(),
                annotations: std::collections::HashMap::new(),
                linux: None,
                windows: None,
            };

 
            if let Ok(mut user_config) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(config.clone()) {
                if let Some(metadata_value) = user_config.get("metadata") {
                    if let Ok(metadata_map) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(metadata_value.clone()) {
                        let mut metadata = crate::api::runtime::v1::PodSandboxMetadata {
                            name: "default-pod".to_string(),
                            uid: uuid::Uuid::new_v4().to_string(),
                            namespace: "default".to_string(),
                            attempt: 0,
                        };
                        
                        if let Some(name) = metadata_map.get("name") {
                            if let Some(name_str) = name.as_str() {
                                metadata.name = name_str.to_string();
                            }
                        }
                        
                        if let Some(uid) = metadata_map.get("uid") {
                            if let Some(uid_str) = uid.as_str() {
                                metadata.uid = uid_str.to_string();
                            }
                        }
                        
                        if let Some(namespace) = metadata_map.get("namespace") {
                            if let Some(namespace_str) = namespace.as_str() {
                                metadata.namespace = namespace_str.to_string();
                            }
                        }
                        
                        if let Some(attempt) = metadata_map.get("attempt") {
                            if let Some(attempt_num) = attempt.as_u64() {
                                metadata.attempt = attempt_num as u32;
                            }
                        }
                        
                        pod_config.metadata = Some(metadata);
                    }
                }
                
       
                if let Some(hostname) = user_config.get("hostname") {
                    if let Some(hostname_str) = hostname.as_str() {
                        pod_config.hostname = hostname_str.to_string();
                    }
                }
                
                if let Some(log_dir) = user_config.get("log_directory") {
                    if let Some(log_dir_str) = log_dir.as_str() {
                        pod_config.log_directory = log_dir_str.to_string();
                    }
                }
                
          
                if let Some(dns_value) = user_config.get("dns_config") {
                    if let Ok(dns_config) = serde_json::from_value::<crate::api::runtime::v1::DnsConfig>(dns_value.clone()) {
                        pod_config.dns_config = Some(dns_config);
                    }
                }
                
             
                if let Some(port_mappings) = user_config.get("port_mappings") {
                    if let Ok(mappings) = serde_json::from_value::<Vec<crate::api::runtime::v1::PortMapping>>(port_mappings.clone()) {
                        pod_config.port_mappings = mappings;
                    }
                }
                
              
                if let Some(labels) = user_config.get("labels") {
                    if let Ok(label_map) = serde_json::from_value::<std::collections::HashMap<String, String>>(labels.clone()) {
                        pod_config.labels = label_map;
                    }
                }
                
              
                if let Some(annotations) = user_config.get("annotations") {
                    if let Ok(anno_map) = serde_json::from_value::<std::collections::HashMap<String, String>>(annotations.clone()) {
                        pod_config.annotations = anno_map;
                    }
                }
                
            
                if let Some(linux_value) = user_config.get("linux") {
                    if let Ok(linux_config) = serde_json::from_value::<crate::api::runtime::v1::LinuxPodSandboxConfig>(linux_value.clone()) {
                        pod_config.linux = Some(linux_config);
                    }
                }
                
            
                if let Some(windows_value) = user_config.get("windows") {
                    if let Ok(windows_config) = serde_json::from_value::<crate::api::runtime::v1::WindowsPodSandboxConfig>(windows_value.clone()) {
                        pod_config.windows = Some(windows_config);
                    }
                }
            } else {
                // if the config is not a map, try to parse it as PodSandboxConfig
                if let Ok(user_pod_config) = serde_json::from_value::<PodSandboxConfig>(config) {
                    // only merge non-empty fields
                    if user_pod_config.metadata.is_some() {
                        pod_config.metadata = user_pod_config.metadata;
                    }
                    
                    if !user_pod_config.hostname.is_empty() {
                        pod_config.hostname = user_pod_config.hostname;
                    }
                    
                    if !user_pod_config.log_directory.is_empty() {
                        pod_config.log_directory = user_pod_config.log_directory;
                    }
                    
                    if user_pod_config.dns_config.is_some() {
                        pod_config.dns_config = user_pod_config.dns_config;
                    }
                    
                    if !user_pod_config.port_mappings.is_empty() {
                        pod_config.port_mappings = user_pod_config.port_mappings;
                    }
                    
                    if !user_pod_config.labels.is_empty() {
                        pod_config.labels = user_pod_config.labels;
                    }
                    
                    if !user_pod_config.annotations.is_empty() {
                        pod_config.annotations = user_pod_config.annotations;
                    }
                    
                    if user_pod_config.linux.is_some() {
                        pod_config.linux = user_pod_config.linux;
                    }
                    
                    if user_pod_config.windows.is_some() {
                        pod_config.windows = user_pod_config.windows;
                    }
                }
            }

            let request = RunPodSandboxRequest {
                config: Some(pod_config),
                runtime_handler: "".to_string(),
            };
            debug!("run pod sandbox request: {:?}", request);

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

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![Prompt::new(
                "check_containerd_status",
                Some("Check if containerd is running"),
                Some(vec![PromptArgument {
                    name: "message".to_string(),
                    description: Some("A message to put in the prompt".to_string()),
                    required: Some(true),
                }]),
            )],
        })
    }
}
