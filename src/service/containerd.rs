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

use crate::api::runtime::v1::PodSandboxMetadata;
use crate::api::runtime::v1::{
    ContainerConfig, CreateContainerRequest, CreateContainerResponse, ImageFsInfoRequest,
    ImageFsInfoResponse, ImageServiceClient, LinuxContainerConfig, LinuxPodSandboxConfig,
    ListContainersRequest, ListContainersResponse, ListImagesRequest, ListImagesResponse,
    ListPodSandboxRequest, ListPodSandboxResponse, Mount, PodSandboxConfig, RemoveContainerRequest,
    RemoveContainerResponse, RemovePodSandboxRequest, RemovePodSandboxResponse,
    RunPodSandboxRequest, RunPodSandboxResponse, RuntimeServiceClient, VersionRequest,
    VersionResponse,
};
use crate::cri::config::{parse_container_config, parse_pod_config};
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

    #[tool(
        description = "List all pod sandboxes created by containerd, showing their status and metadata"
    )]
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

    #[tool(
        description = "List all containers managed by containerd, including their status, pod association, and metadata"
    )]
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

    #[tool(
        description = "List all container images available in the containerd registry, including their tags, digests, and sizes"
    )]
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

    #[tool(
        description = "Get filesystem information for container images, including storage capacity and usage metrics"
    )]
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

    #[tool(
        description = "Create a new pod sandbox with customizable configuration including networking, security settings, and resource constraints"
    )]
    pub async fn create_pod(
        &self,
        #[tool(param)]
        #[schemars(description = "
            For creating a pod config, the example is:
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
            ")]
        config: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        debug!("create pod request: {:?}", config);
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // Parse pod configuration with defaults
            let pod_config = parse_pod_config(config);

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

    #[tool(
        description = "Remove a pod sandbox and clean up all associated resources, including network namespaces"
    )]
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
    /// sandbox_config is none will cause containerd panic
    /// fix it , and use more greater method to create container
    #[tool(
        description = "Create a new container within a pod sandbox with configurable runtime settings, environment variables, mounts, and image specification"
    )]
    pub async fn create_container(
        &self,
        #[tool(param)]
        #[schemars(description = "
            For creating a container config, the example is:
            {
                \"pod_id\": \"pod-12345\", 
                \"config\": {
                    \"metadata\": {
                        \"name\": \"my-container\"
                    }, 
                    \"image\": {
                        \"image\": \"nginx:latest\"
                    }, 
                    \"command\": [\"/bin/sh\"], 
                    \"args\": [\"-c\", \"while true; do echo hello; sleep 10; done\"],
                    \"working_dir\": \"/\",
                    \"envs\": [
                        {
                            \"key\": \"PATH\",
                            \"value\": \"/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\"
                        }
                    ],
                    \"labels\": {
                        \"app\": \"nginx\"
                    },
                    \"annotations\": {
                        \"key\": \"value\"
                    },
                    \"log_path\": \"my-container/0.log\",
                    \"stdin\": false,
                    \"stdin_once\": false,
                    \"tty\": false
                }
            }")]
        params: serde_json::Value,
        #[tool(param)]
        #[schemars(
            description = "Optional pod sandbox configuration to use when creating the container. Provides context for container creation within the pod."
        )]
        pod_config: Option<serde_json::Value>,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // Get pod_id from params
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

            // Get container configuration from params
            let container_config = match params.get("config") {
                Some(config) => parse_container_config(config.clone()),
                None => {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Missing required parameter: config",
                    )]));
                }
            };

            // Parse pod configuration for sandbox_config
            let sandbox_config = pod_config.map(parse_pod_config);

            let request = CreateContainerRequest {
                pod_sandbox_id: pod_id,
                config: Some(container_config),
                sandbox_config,
            };

            debug!("create container request: {:?}", request);

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

    #[tool(
        description = "Remove a container from a pod sandbox and clean up all associated resources, including filesystem mounts"
    )]
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

    #[tool(description = "Stop a running pod sandbox and all its containers")]
    pub async fn stop_pod(
        &self,
        #[tool(param)]
        #[schemars(
            description = "JSON containing the pod_id to identify which pod sandbox to stop"
        )]
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // Get pod id from json
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

            let request = crate::api::runtime::v1::StopPodSandboxRequest {
                pod_sandbox_id: pod_id,
            };

            match client.clone().stop_pod_sandbox(request).await {
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
        #[schemars(
            description = "JSON containing the container_id to identify which container to start"
        )]
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // Get container id from json
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

            let request = crate::api::runtime::v1::StartContainerRequest { container_id };

            match client.clone().start_container(request).await {
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
        #[schemars(
            description = "JSON containing the container_id to identify which container to stop and an optional timeout in seconds"
        )]
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // Get container id from json
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

            // Get timeout if provided (default: 0)
            let timeout = match params.get("timeout") {
                Some(timeout) => match timeout.as_i64() {
                    Some(t) => t as i64,
                    None => 0,
                },
                None => 0,
            };

            let request = crate::api::runtime::v1::StopContainerRequest {
                container_id,
                timeout,
            };

            match client.clone().stop_container(request).await {
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

    #[tool(description = "Execute a command in a running container with optional TTY and stdin")]
    pub async fn exec(
        &self,
        #[tool(param)]
        #[schemars(
            description = "JSON containing the container_id and command to execute, with optional TTY and stdin settings"
        )]
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // Get container id
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

            // Get command
            let command = match params.get("command") {
                Some(cmd) => match cmd.as_array() {
                    Some(cmd_arr) => {
                        let mut cmd_vec = Vec::new();
                        for c in cmd_arr {
                            if let Some(c_str) = c.as_str() {
                                cmd_vec.push(c_str.to_string());
                            } else {
                                return Ok(CallToolResult::error(vec![Content::text(
                                    "Command array must contain only strings",
                                )]));
                            }
                        }
                        cmd_vec
                    }
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "command must be an array of strings",
                        )]));
                    }
                },
                None => {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Missing required parameter: command",
                    )]));
                }
            };

            // Get tty (optional)
            let tty = match params.get("tty") {
                Some(t) => match t.as_bool() {
                    Some(t_bool) => t_bool,
                    None => false,
                },
                None => false,
            };

            // Get stdin (optional)
            let stdin = match params.get("stdin") {
                Some(s) => match s.as_bool() {
                    Some(s_bool) => s_bool,
                    None => false,
                },
                None => false,
            };

            let request = crate::api::runtime::v1::ExecSyncRequest {
                container_id,
                cmd: command,
                timeout: 10, // Default timeout of 10 seconds
            };

            match client.clone().exec_sync(request).await {
                Ok(response) => {
                    let response_inner = response.into_inner();
                    let stdout = String::from_utf8_lossy(&response_inner.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&response_inner.stderr).to_string();
                    let exit_code = response_inner.exit_code;

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

    #[tool(
        description = "Pull an image from a registry to make it available for container creation"
    )]
    pub async fn pull_image(
        &self,
        #[tool(param)]
        #[schemars(
            description = "JSON containing the image reference to pull and optional authentication credentials"
        )]
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.image_client.lock().await;
        if let Some(client) = &*lock {
            // Get image reference
            let image = match params.get("image") {
                Some(img) => match img.as_str() {
                    Some(img_str) => img_str.to_string(),
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "image must be a string",
                        )]));
                    }
                },
                None => {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Missing required parameter: image",
                    )]));
                }
            };

            // Get auth config if provided
            let auth = match params.get("auth") {
                Some(auth) => {
                    let auth_map = auth.as_object();
                    match auth_map {
                        Some(map) => {
                            let username =
                                map.get("username").and_then(|u| u.as_str()).unwrap_or("");
                            let password =
                                map.get("password").and_then(|p| p.as_str()).unwrap_or("");
                            let auth = map.get("auth").and_then(|a| a.as_str()).unwrap_or("");
                            let server_address = map
                                .get("server_address")
                                .and_then(|s| s.as_str())
                                .unwrap_or("");
                            let identity_token = map
                                .get("identity_token")
                                .and_then(|i| i.as_str())
                                .unwrap_or("");

                            Some(crate::api::runtime::v1::AuthConfig {
                                username: username.to_string(),
                                password: password.to_string(),
                                auth: auth.to_string(),
                                server_address: server_address.to_string(),
                                identity_token: identity_token.to_string(),
                                registry_token: "".to_string(),
                            })
                        }
                        None => None,
                    }
                }
                None => None,
            };

            let request = crate::api::runtime::v1::PullImageRequest {
                image: Some(crate::api::runtime::v1::ImageSpec {
                    image,
                    annotations: std::collections::HashMap::new(),
                    runtime_handler: "".to_string(),
                    user_specified_image: "".to_string(),
                }),
                auth,
                sandbox_config: None,
            };

            match client.clone().pull_image(request).await {
                Ok(response) => {
                    let image_ref = response.into_inner().image_ref;
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
        #[schemars(description = "JSON containing the image reference to remove")]
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.image_client.lock().await;
        if let Some(client) = &*lock {
            // Get image reference
            let image = match params.get("image") {
                Some(img) => match img.as_str() {
                    Some(img_str) => img_str.to_string(),
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "image must be a string",
                        )]));
                    }
                },
                None => {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Missing required parameter: image",
                    )]));
                }
            };

            let request = crate::api::runtime::v1::RemoveImageRequest {
                image: Some(crate::api::runtime::v1::ImageSpec {
                    image,
                    annotations: std::collections::HashMap::new(),
                    runtime_handler: "".to_string(),
                    user_specified_image: "".to_string(),
                }),
            };

            match client.clone().remove_image(request).await {
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
        #[schemars(
            description = "JSON containing the container_id, tail lines, follow option, and timestamps option"
        )]
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // Get container id
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

            // Get tail lines option
            let tail_lines = match params.get("tail") {
                Some(t) => match t.as_i64() {
                    Some(t_int) => t_int as i64,
                    None => 100, // Default to 100 lines
                },
                None => 100, // Default to 100 lines
            };

            let request = crate::api::runtime::v1::ContainerStatusRequest {
                container_id: container_id.clone(),
                verbose: true,
            };

            match client.clone().container_status(request).await {
                Ok(status_response) => {
                    let status = status_response.into_inner();

                    // Get the log path from the container status
                    let log_path = match status.status {
                        Some(container_status) => {
                            format!(
                                "{}/{}",
                                status.info.get("sandboxLogDir").unwrap_or(&"".to_string()),
                                container_status.log_path
                            )
                        }
                        None => {
                            return Ok(CallToolResult::error(vec![Content::text(
                                "Container status not available",
                            )]));
                        }
                    };

                    // Read the log file
                    match std::fs::read_to_string(&log_path) {
                        Ok(log_content) => {
                            let mut lines: Vec<&str> = log_content.lines().collect();

                            // Apply tail if needed
                            if tail_lines > 0 && (tail_lines as usize) < lines.len() {
                                lines = lines[(lines.len() - tail_lines as usize)..].to_vec();
                            }

                            // Join lines with newline
                            let filtered_content = lines.join("\n");

                            return Ok(CallToolResult::success(vec![Content::text(
                                filtered_content,
                            )]));
                        }
                        Err(e) => {
                            return Ok(CallToolResult::error(vec![Content::text(format!(
                                "Failed to read container logs at {}: {}",
                                log_path, e
                            ))]));
                        }
                    }
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to get container status: {}",
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
        #[schemars(description = "JSON containing the container_id to retrieve statistics for")]
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // Get container id
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

            let request = crate::api::runtime::v1::ContainerStatsRequest { container_id };

            match client.clone().container_stats(request).await {
                Ok(response) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        serde_json::to_string(&response.into_inner()).unwrap(),
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
        #[schemars(description = "JSON containing optional filter for pod_id to limit results")]
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            // Get optional pod id filter
            let pod_id = match params.get("pod_id") {
                Some(id) => match id.as_str() {
                    Some(id_str) => Some(id_str.to_string()),
                    None => None,
                },
                None => None,
            };

            // Create filter if pod_id is provided
            let filter = match pod_id {
                Some(id) => Some(crate::api::runtime::v1::PodSandboxStatsFilter {
                    id,
                    label_selector: std::collections::HashMap::new(),
                }),
                None => None,
            };

            let request = crate::api::runtime::v1::ListPodSandboxStatsRequest { filter };

            match client.clone().list_pod_sandbox_stats(request).await {
                Ok(response) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        serde_json::to_string(&response.into_inner()).unwrap(),
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
            instructions: Some("This server provides tools to interact with the Container Runtime Interface (CRI) of Containerd. You can manage container lifecycle including creating and removing pods and containers, listing existing resources, and querying runtime information. Available tools: 'version' for runtime version; 'list_pods', 'list_containers', 'list_images', 'image_fs_info' for resource listing; 'create_pod', 'stop_pod', and 'remove_pod' for pod management; 'create_container', 'start_container', 'stop_container', 'exec', and 'remove_container' for container management; 'pull_image' and 'remove_image' for image management; 'container_stats', 'pod_stats', and 'container_logs' for monitoring. Use these tools to build and manage containerized applications through the CRI standard interface.".to_string()),
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
