/*
 * Containerd Service Implementation - Container Runtime Interface (CRI)
 *
 * This service provides tools to interact with Containerd through the Container Runtime Interface (CRI).
 * CRI is a plugin interface which enables kubelet to use different container runtimes.
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

use anyhow::Result;
use rmcp::{
    const_string, model::*, schemars, service::RequestContext, tool, Error as McpError, RoleServer,
    ServerHandler,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

#[derive(Clone)]
pub struct Server {
    endpoint: String,
    runtime_client: Arc<Mutex<Option<crate::api::runtime::v1::RuntimeServiceClient<tonic::transport::Channel>>>>,
    image_client: Arc<Mutex<Option<crate::api::runtime::v1::ImageServiceClient<tonic::transport::Channel>>>>,
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
            *lock = Some(crate::api::runtime::v1::RuntimeServiceClient::new(channel.clone()));
        }

        {
            debug!("connect image client");
            let mut lock = self.image_client.lock().await;
            *lock = Some(crate::api::runtime::v1::ImageServiceClient::new(channel));
        }

        Ok(())
    }

    /// This is a test for mcp params
    #[tool(description = "Test for mcp params")]
    pub async fn test_mcp_params(
        &self,
        #[tool(param)]
        #[schemars(description = "The param to test")]
        param: bool,
    ) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Test for mcp params: {}",
            param
        ))]))
    }

    /// This interface may have some security issues, need to be fixed
    #[tool(description = "Get the containerd log file contents to diagnose runtime issues")]
    pub async fn get_containerd_logs(
        &self,
        #[tool(param)]
        #[schemars(
            description = "The path to the containerd log file, default is /var/log/containerd/containerd.log"
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

    #[tool(
        description = "Get version information from the containerd runtime to verify compatibility"
    )]
    pub async fn version(&self) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::runtime::version(&mut client_clone).await {
                Ok(version_response) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        serde_json::to_string(&version_response).unwrap(),
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to get version: {}",
                        e
                    ))]));
                }
            }
        }
        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    #[tool(
        description = "List all pod sandboxes created by containerd, showing their status and metadata"
    )]
    pub async fn list_pods(&self) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::pod::list_pods(&mut client_clone).await {
                Ok(response) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        serde_json::to_string(&response).unwrap(),
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to list pods: {}",
                        e
                    ))]));
                }
            }
        }
        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    #[tool(
        description = "List all containers managed by containerd, including their status, pod association, and metadata"
    )]
    pub async fn list_containers(&self) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let request = crate::api::runtime::v1::ListContainersRequest { filter: None };
            let response = client.clone().list_containers(request).await.unwrap();
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string(&response.into_inner()).unwrap(),
            )]));
        }
        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    #[tool(
        description = "List all container images available in the containerd registry, including their tags, digests, and sizes"
    )]
    pub async fn list_images(&self) -> Result<CallToolResult, McpError> {
        let lock = self.image_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::image::list_images(&mut client_clone).await {
                Ok(response) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        serde_json::to_string(&response).unwrap(),
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to list images: {}",
                        e
                    ))]));
                }
            }
        }
        Ok(CallToolResult::error(vec![Content::text(
            "Image client not connected",
        )]))
    }

    #[tool(
        description = "Get filesystem information for container images, including storage capacity and usage metrics"
    )]
    pub async fn image_fs_info(&self) -> Result<CallToolResult, McpError> {
        let lock = self.image_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::image::image_fs_info(&mut client_clone).await {
                Ok(response) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        serde_json::to_string(&response).unwrap(),
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to get image filesystem info: {}",
                        e
                    ))]));
                }
            }
        }
        Ok(CallToolResult::error(vec![Content::text(
            "Image client not connected",
        )]))
    }

    #[tool(
        description = "Create a new pod sandbox with customizable configuration including networking, security settings, and resource constraints"
    )]
    pub async fn create_pod(
        &self,
        #[tool(param)]
        #[schemars(
            description = "Pod name - a unique identifier for the pod within its namespace"
        )]
        name: String,

        #[tool(param)]
        #[schemars(description = "Namespace for the pod (e.g., 'default', 'kube-system')")]
        namespace: String,

        #[tool(param)]
        #[schemars(description = "Unique identifier for the pod (UUID format recommended)")]
        uid: String,

        #[tool(param)]
        #[schemars(
            description = "Additional pod configuration options in hashmap format,the format is json in string, including:
            - hostname: Custom hostname for the pod
            - attempt: Pod creation attempt count (default: 0)
            - log_directory: Path to store container logs
            - dns_config: DNS server configuration (Example: {\"servers\": [\"8.8.8.8\"], \"searches\": [\"example.com\"], \"options\": [\"ndots:2\"]})
            - port_mappings: Container port to host port mappings (Example: [{\"protocol\": \"TCP\", \"container_port\": 80, \"host_port\": 8080}])
            - labels: Key-value pairs for pod identification (Example: {\"app\": \"nginx\"})
            - annotations: Unstructured metadata as key-value pairs (Example: {\"key\": \"value\"})
            - linux: Linux-specific configurations 
            - windows: Windows-specific configurations
            
            Example options: {
                \"hostname\": \"custom-host\",
                \"log_directory\": \"/custom/log/path\",
                \"labels\": {\"app\": \"nginx\", \"environment\": \"production\"},
                \"dns_config\": {\"servers\": [\"8.8.8.8\", \"1.1.1.1\"]}
            }"
        )]
        options: String,
    ) -> Result<CallToolResult, McpError> {
        debug!(
            "Create pod request - name: {}, namespace: {}, uid: {}, options: {:?}",
            name, namespace, uid, options
        );
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::pod::create_pod(&mut client_clone, name, namespace, uid, options).await {
                Ok((pod_id, pod_config)) => {
                    let create_pod_result = serde_json::json!({
                        "pod_id": pod_id,
                        "pod_config": pod_config
                    });
                    return Ok(CallToolResult::success(vec![Content::text(
                        serde_json::to_string(&create_pod_result).unwrap(),
                    )]));
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

    #[tool(
        description = "Remove a pod sandbox and clean up all associated resources, including network namespaces"
    )]
    pub async fn remove_pod(
        &self,
        #[tool(param)]
        #[schemars(description = "The pod id to remove")]
        pod_id: String,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::pod::remove_pod(&mut client_clone, pod_id).await {
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
    /// sandbox_config is none will cause containerd panic
    /// fix it , and use more greater method to create container
    #[tool(
        description = "Create a new container within a pod sandbox with configurable runtime settings, environment variables, mounts, and image specification"
    )]
    pub async fn create_container(
        &self,
        #[tool(param)]
        #[schemars(description = "Pod ID that this container will run in")]
        pod_id: String,

        #[tool(param)]
        #[schemars(
            description = "Container name - a unique identifier for the container within its pod"
        )]
        name: String,

        #[tool(param)]
        #[schemars(description = "Container image to use (e.g., 'nginx:latest', 'ubuntu:20.04')")]
        image: String,

        #[tool(param)]
        #[schemars(
            description = "Additional container configuration options in hashmap format,the format is json in string, including:
            - command: Command to execute in the container (array of strings)
            - args: Arguments to the command (array of strings)
            - working_dir: Working directory for the command
            - envs: Environment variables as key-value pairs (Example: [{\"key\": \"PATH\", \"value\": \"/usr/local/sbin:/usr/bin\"}])
            - labels: Key-value pairs for container identification (Example: {\"app\": \"nginx\"})
            - annotations: Unstructured metadata as key-value pairs (Example: {\"key\": \"value\"})
            - mounts: Volume mounts (Example: [{\"host_path\": \"/host/path\", \"container_path\": \"/container/path\", \"readonly\": false}])
            - log_path: Path for container logs relative to the pod log directory
            - stdin: Whether to keep stdin open (boolean)
            - stdin_once: Whether to close stdin after first attach (boolean)
            - tty: Whether to allocate a TTY (boolean)
            - linux: Linux-specific configurations
            - windows: Windows-specific configurations
            
            Example options: {
                \"command\": [\"/bin/sh\"],
                \"args\": [\"-c\", \"while true; do echo hello; sleep 10; done\"],
                \"working_dir\": \"/app\",
                \"envs\": [{\"key\": \"DEBUG\", \"value\": \"true\"}],
                \"labels\": {\"component\": \"web\", \"tier\": \"frontend\"}
            }"
        )]
        options: String,

        #[tool(param)]
        #[schemars(
            description = "It must be the result of create_pod tool, provides context for container creation within the pod, the format is json in string"
        )]
        pod_config: String,
    ) -> Result<CallToolResult, McpError> {
        debug!(
            "Create container request - pod_id: {}, name: {}, image: {}, options: {:?}",
            pod_id, name, image, options
        );
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::container::create_container(
                &mut client_clone, 
                pod_id, 
                name, 
                image, 
                options, 
                pod_config
            ).await {
                Ok(container_id) => {
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

    #[tool(
        description = "Remove a container from a pod sandbox and clean up all associated resources, including filesystem mounts"
    )]
    pub async fn remove_container(
        &self,
        #[tool(param)]
        #[schemars(description = "The container id to remove")]
        container_id: String,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::container::remove_container(&mut client_clone, container_id).await {
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

    #[tool(description = "Stop a running pod sandbox and all its containers")]
    pub async fn stop_pod(
        &self,
        #[tool(param)]
        #[schemars(description = "The pod id to stop")]
        pod_id: String,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::pod::stop_pod(&mut client_clone, pod_id).await {
                Ok(_) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "{\"success\": true, \"message\": \"Pod stopped successfully\"}",
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to stop pod: {}",
                        e
                    ))]));
                }
            }
        }

        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    #[tool(description = "Start a created container, making it ready to execute workloads")]
    pub async fn start_container(
        &self,
        #[tool(param)]
        #[schemars(description = "The container id to start")]
        container_id: String,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::container::start_container(&mut client_clone, container_id).await {
                Ok(_) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "{\"success\": true, \"message\": \"Container started successfully\"}",
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to start container: {}",
                        e
                    ))]));
                }
            }
        }

        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    #[tool(description = "Stop a running container gracefully with an optional timeout")]
    pub async fn stop_container(
        &self,
        #[tool(param)]
        #[schemars(description = "The container id to stop")]
        id: String,
        #[tool(param)]
        #[schemars(description = "Timeout in seconds for container stop (default: 10)")]
        timeout: i64,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::container::stop_container(&mut client_clone, id, timeout).await {
                Ok(_) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "{\"success\": true, \"message\": \"Container stopped successfully\"}",
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to stop container: {}",
                        e
                    ))]));
                }
            }
        }

        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    #[tool(description = "Execute a command in a running container in sync mode")]
    pub async fn exec_sync(
        &self,
        #[tool(param)]
        #[schemars(description = "The container id to execute the command in")]
        container_id: String,

        #[tool(param)]
        #[schemars(description = "The command to execute")]
        command: String,

        #[tool(param)]
        #[schemars(
            description = "Optional timeout in seconds for command execution (default: 10)"
        )]
        timeout: Option<i64>,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::container::exec_sync(&mut client_clone, container_id, command, timeout).await {
                Ok(response) => {
                    let stdout = String::from_utf8_lossy(&response.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&response.stderr).to_string();
                    let exit_code = response.exit_code;

                    let result = serde_json::json!({
                        "stdout": stdout,
                        "stderr": stderr,
                        "exit_code": exit_code
                    });

                    return Ok(CallToolResult::success(vec![Content::text(
                        result.to_string(),
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to execute command: {}",
                        e
                    ))]));
                }
            }
        }

        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    /// Now not support pull with auth
    #[tool(
        description = "Pull an image from a registry to make it available for container creation"
    )]
    pub async fn pull_image(
        &self,
        #[tool(param)]
        #[schemars(
            description = "The image reference to pull, e.g. docker.io/library/nginx:latest"
        )]
        image_reference: String,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.image_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::image::pull_image(&mut client_clone, image_reference.clone()).await {
                Ok(image_ref) => {
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "{{\"success\": true, \"image_ref\": \"{}\"}}",
                        image_ref
                    ))]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to pull image: {}",
                        e
                    ))]));
                }
            }
        }

        Ok(CallToolResult::error(vec![Content::text(
            "Image client not connected",
        )]))
    }

    #[tool(description = "Remove an image from the container runtime to free up disk space")]
    pub async fn remove_image(
        &self,
        #[tool(param)]
        #[schemars(
            description = "The image reference to remove, e.g. docker.io/library/nginx:latest"
        )]
        image_reference: String,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.image_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::image::remove_image(&mut client_clone, image_reference.clone()).await {
                Ok(_) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "{\"success\": true, \"message\": \"Image removed successfully\"}",
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to remove image: {}",
                        e
                    ))]));
                }
            }
        }

        Ok(CallToolResult::error(vec![Content::text(
            "Image client not connected",
        )]))
    }

    #[tool(
        description = "Retrieve logs from a container with optional timestamp, tail lines, and follow options"
    )]
    pub async fn container_logs(
        &self,
        #[tool(param)]
        #[schemars(description = "The container id to retrieve logs from")]
        container_id: String,

        #[tool(param)]
        #[schemars(description = "Optional tail lines to retrieve (default: 100)")]
        tail: Option<i64>,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::container::container_logs(&mut client_clone, container_id).await {
                Ok((log_content, log_path)) => {
                    let mut lines: Vec<&str> = log_content.lines().collect();

                    // Apply tail if needed
                    if let Some(tail_lines) = tail {
                        let tail_count = std::cmp::min(tail_lines as usize, lines.len());
                        if tail_count > 0 {
                            lines = lines[(lines.len() - tail_count)..].to_vec();
                        }
                    }

                    // Join lines with newline
                    let filtered_content = lines.join("\n");

                    return Ok(CallToolResult::success(vec![Content::text(
                        filtered_content,
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to get container logs: {}",
                        e
                    ))]));
                }
            }
        }

        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    #[tool(description = "Get detailed resource usage statistics for a container")]
    pub async fn container_stats(
        &self,
        #[tool(param)]
        #[schemars(description = "The container id to retrieve statistics for")]
        container_id: String,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::container::container_stats(&mut client_clone, container_id).await {
                Ok(response) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        serde_json::to_string(&response).unwrap(),
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to get container stats: {}",
                        e
                    ))]));
                }
            }
        }

        Ok(CallToolResult::error(vec![Content::text(
            "Runtime client not connected",
        )]))
    }

    #[tool(description = "Get aggregate resource usage statistics for all pods")]
    pub async fn pod_stats(
        &self,
        #[tool(param)]
        #[schemars(description = "Optional pod id to retrieve stats for")]
        pod_id: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::pod::pod_stats(&mut client_clone, pod_id).await {
                Ok(response) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        serde_json::to_string(&response).unwrap(),
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to get pod stats: {}",
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
            instructions: Some("This server provides tools to interact with the Container Runtime Interface (CRI) of Containerd. You can manage container lifecycle including creating and removing pods and containers, listing existing resources, and querying runtime information. Available tools: 'version' for runtime version; 'list_pods', 'list_containers', 'list_images', 'image_fs_info' for resource listing; 'create_pod' (with name, namespace, uid and options parameters) for pod creation; 'stop_pod', and 'remove_pod' for pod management; 'create_container' (with pod_id, name, image and options parameters) for container creation; 'start_container', 'stop_container', 'exec', and 'remove_container' for container management; 'pull_image' and 'remove_image' for image management; 'container_stats', 'pod_stats', and 'container_logs' for monitoring. Use these tools to build and manage containerized applications through the CRI standard interface.".to_string()),
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
