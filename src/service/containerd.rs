/*
 * Containerd Service Implementation - Container Runtime Interface (CRI) and CTR
 *
 * This service provides tools to interact with Containerd through both:
 * 1. Container Runtime Interface (CRI) - Standard K8s container runtime interface
 * 2. CTR CLI tool - Direct containerd command line interface
 *
 * CRI Tool Interfaces:
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
 *
 * CTR Tool Interfaces:
 * - run_ctr_command: Run any ctr command
 * - list_containers_ctr: List all containers using ctr
 * - list_images_ctr: List all images using ctr
 * - list_tasks_ctr: List all tasks using ctr
 * - pull_image_ctr: Pull an image using ctr
 * - remove_image_ctr: Remove an image using ctr
 * - run_container_ctr: Run a container using ctr
 * - remove_container_ctr: Remove a container using ctr
 */
#![allow(dead_code)]
use crate::ctr::cmd::CtrCmd;
use anyhow::Result;
use rmcp::{
    handler::server::tool::{Parameters, ToolRouter}, model::*, schemars, service::RequestContext, tool, tool_router,tool_handler, Error as McpError, RoleServer, ServerHandler
};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunCtrCommandParams {
    #[schemars(
        description = "The ctr command to run, e.g. 'container list', 'image pull <image>'"
    )]
    command: String,
    #[schemars(description = "The namespace to use for the ctr command")]
    namespace: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListContainersCtrParams {
    #[schemars(description = "The namespace to use for the ctr command")]
    namespace: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListImagesCtrParams {
    #[schemars(description = "The namespace to use for the ctr command")]
    namespace: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListTasksCtrParams {
    #[schemars(description = "The namespace to use for the ctr command")]
    namespace: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PullImageCtrParams {
    #[schemars(description = "The image reference to pull, e.g. 'docker.io/library/nginx:latest'")]
    image_reference: String,
    #[schemars(description = "The namespace to use for the ctr command")]
    namespace: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoveImageCtrParams {
    #[schemars(description = "The image reference to remove, e.g. 'docker.io/library/nginx:latest'")]
    image_reference: String,
    #[schemars(description = "The namespace to use for the ctr command")]
    namespace: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunContainerCtrParams {
    #[schemars(description = "The image reference to use, e.g. 'docker.io/library/nginx:latest'")]
    image_reference: String,
    #[schemars(description = "The container ID or name")]
    container_id: String,
    #[schemars(description = "Additional arguments for the container run command (as a space-separated string)")]
    args: String,
    #[schemars(description = "The namespace to use for the ctr command")]
    namespace: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoveContainerCtrParams {
    #[schemars(description = "The container ID or name to remove")]
    container_id: String,
    #[schemars(description = "The namespace to use for the ctr command")]
    namespace: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetContainerdLogsParams {
    #[schemars(description = "The path to the containerd log file, default is /var/log/containerd/containerd.log")]
    path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReopenContainerLogParams {
    #[schemars(description = "The container id to reopen the log for")]
    container_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreatePodParams {
    #[schemars(description = "Pod name - a unique identifier for the pod within its namespace")]
    name: String,
    #[schemars(description = "Namespace for the pod (e.g., 'default', 'kube-system')")]
    namespace: String,
    #[schemars(description = "Unique identifier for the pod (UUID format recommended)")]
    uid: String,
    #[schemars(description = "Additional pod configuration options in hashmap format,the format is json in string")]
    options: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RemovePodParams {
    #[schemars(description = "The pod id to remove")]
    pod_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateContainerParams {
    #[schemars(description = "Pod ID that this container will run in")]
    pod_id: String,
    #[schemars(description = "Container name - a unique identifier for the container within its pod")]
    name: String,
    #[schemars(description = "Container image to use (e.g., 'nginx:latest', 'ubuntu:20.04')")]
    image: String,
    #[schemars(description = "Additional container configuration options in hashmap format,the format is json in string")]
    options: String,
    #[schemars(description = "It must be the result of create_pod tool, provides context for container creation within the pod, the format is json in string")]
    pod_config: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoveContainerParams {
    #[schemars(description = "The container id to remove")]
    container_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StopPodParams {
    #[schemars(description = "The pod id to stop")]
    pod_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StartContainerParams {
    #[schemars(description = "The container id to start")]
    container_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StopContainerParams {
    #[schemars(description = "The container id to stop")]
    id: String,
    #[schemars(description = "Timeout in seconds for container stop (default: 10)")]
    timeout: i64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExecSyncParams {
    #[schemars(description = "The container id to execute the command in")]
    container_id: String,
    #[schemars(description = "The command to execute")]
    command: String,
    #[schemars(description = "Optional timeout in seconds for command execution (default: 10)")]
    timeout: Option<i64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PullImageParams {
    #[schemars(description = "The image reference to pull, e.g. docker.io/library/nginx:latest")]
    image_reference: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoveImageParams {
    #[schemars(description = "The image reference to remove, e.g. docker.io/library/nginx:latest")]
    image_reference: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ContainerLogsParams {
    #[schemars(description = "The container id to retrieve logs from")]
    container_id: String,
    #[schemars(description = "Optional tail lines to retrieve (default: 100)")]
    tail: Option<i64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ContainerStatsParams {
    #[schemars(description = "The container id to retrieve statistics for")]
    container_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PodStatsParams {
    #[schemars(description = "Optional pod id to retrieve stats for")]
    pod_id: Option<String>,
}

type RuntimeClient = Arc<
    Mutex<Option<crate::api::runtime::v1::RuntimeServiceClient<tonic::transport::Channel>>>,
>;
type ImageClient = Arc<
    Mutex<Option<crate::api::runtime::v1::ImageServiceClient<tonic::transport::Channel>>>,
>;
#[derive(Clone)]
pub struct Server {
    endpoint: String,
    runtime_client: RuntimeClient,
    image_client: ImageClient,
    binary: String,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Server {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            runtime_client: Arc::new(Mutex::new(None)),
            image_client: Arc::new(Mutex::new(None)),
            binary: "ctr".to_string(),
            tool_router: Self::tool_router(),
        }
    }

    /// Helper function to create a CtrCmd instance
    fn create_ctr_cmd(&self, namespace: String) -> CtrCmd {
        CtrCmd::with_config(self.binary.clone(), namespace)
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
            *lock = Some(crate::api::runtime::v1::RuntimeServiceClient::new(
                channel.clone(),
            ));
        }

        {
            debug!("connect image client");
            let mut lock = self.image_client.lock().await;
            *lock = Some(crate::api::runtime::v1::ImageServiceClient::new(channel));
        }

        Ok(())
    }

    // ================== CTR Tool Functions ==================
    #[tool(description = "Run any ctr command with custom arguments")]
    pub async fn run_ctr_command(
        &self,
        Parameters(RunCtrCommandParams { command, namespace }): Parameters<RunCtrCommandParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Running ctr command: {}", command);

        // Split the command into parts
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Command cannot be empty",
            )]));
        }

        let ctr_cmd = self.create_ctr_cmd(namespace);
        debug!("Created ctr command: {:?}", ctr_cmd);
        match ctr_cmd.custom_command(parts[0], parts[1..].to_vec()) {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                let result = format!(
                    "Exit Code: {}\n\nStdout:\n{}\n\nStderr:\n{}",
                    output.status.code().unwrap_or(-1),
                    stdout,
                    stderr
                );

                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to execute ctr command: {}",
                e
            ))])),
        }
    }

    #[tool(description = "List all containers using ctr command")]
    pub async fn list_containers_ctr(
        &self,
        Parameters(ListContainersCtrParams { namespace }): Parameters<ListContainersCtrParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Listing containers with ctr");

        let ctr_cmd = self.create_ctr_cmd(namespace);
        match ctr_cmd.containers_list() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(CallToolResult::success(vec![Content::text(stdout)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to list containers: {}",
                e
            ))])),
        }
    }

    #[tool(description = "List all images using ctr command")]
    pub async fn list_images_ctr(
        &self,
        Parameters(ListImagesCtrParams { namespace }): Parameters<ListImagesCtrParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Listing images with ctr");

        let ctr_cmd = self.create_ctr_cmd(namespace);
        match ctr_cmd.images_list() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(CallToolResult::success(vec![Content::text(stdout)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to list images: {}",
                e
            ))])),
        }
    }

    #[tool(description = "List all tasks (running containers) using ctr command")]
    pub async fn list_tasks_ctr(
        &self,
        Parameters(ListTasksCtrParams { namespace }): Parameters<ListTasksCtrParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Listing tasks with ctr");

        let ctr_cmd = self.create_ctr_cmd(namespace);
        match ctr_cmd.tasks_list() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(CallToolResult::success(vec![Content::text(stdout)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to list tasks: {}",
                e
            ))])),
        }
    }

    #[tool(description = "Pull an image using ctr command")]
    pub async fn pull_image_ctr(
        &self,
        Parameters(PullImageCtrParams { image_reference, namespace }): Parameters<PullImageCtrParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Pulling image with ctr: {}", image_reference);

        let ctr_cmd = self.create_ctr_cmd(namespace);
        match ctr_cmd.image_pull(&image_reference) {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Successfully pulled image: {}\n\n{}",
                        image_reference, stdout
                    ))]))
                } else {
                    Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to pull image: {}\n\n{}",
                        image_reference, stderr
                    ))]))
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to execute pull command: {}",
                e
            ))])),
        }
    }

    #[tool(description = "Remove an image using ctr command")]
    pub async fn remove_image_ctr(
        &self,
        Parameters(RemoveImageCtrParams { image_reference, namespace }): Parameters<RemoveImageCtrParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Removing image with ctr: {}", image_reference);

        let ctr_cmd = self.create_ctr_cmd(namespace);
        match ctr_cmd.image_remove(&image_reference) {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Successfully removed image: {}\n\n{}",
                        image_reference, stdout
                    ))]))
                } else {
                    Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to remove image: {}\n\n{}",
                        image_reference, stderr
                    ))]))
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to execute remove command: {}",
                e
            ))])),
        }
    }

    #[tool(description = "Run a container using ctr command")]
    pub async fn run_container_ctr(
        &self,
        Parameters(RunContainerCtrParams { image_reference, container_id, args, namespace }): Parameters<RunContainerCtrParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(
            "Running container with ctr - image: {}, id: {}, args: {}",
            image_reference, container_id, args
        );

        let args_vec: Vec<String> = args.split_whitespace().map(|s| s.to_string()).collect();

        let ctr_cmd = self.create_ctr_cmd(namespace);
        match ctr_cmd.container_run(&image_reference, &container_id, args_vec) {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Successfully created container: {}\n\n{}",
                        container_id, stdout
                    ))]))
                } else {
                    Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to create container: {}\n\n{}",
                        container_id, stderr
                    ))]))
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to execute container run command: {}",
                e
            ))])),
        }
    }

    #[tool(description = "Remove a container using ctr command")]
    pub async fn remove_container_ctr(
        &self,
        Parameters(RemoveContainerCtrParams { container_id, namespace }): Parameters<RemoveContainerCtrParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Removing container with ctr: {}", container_id);

        let ctr_cmd = self.create_ctr_cmd(namespace);
        match ctr_cmd.container_remove(&container_id) {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Successfully removed container: {}\n\n{}",
                        container_id, stdout
                    ))]))
                } else {
                    Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to remove container: {}\n\n{}",
                        container_id, stderr
                    ))]))
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to execute remove container command: {}",
                e
            ))])),
        }
    }

    // ================== CRI Tool Functions ==================

    /// This interface may have some security issues, need to be fixed
    #[tool(description = "Get the containerd log file contents to diagnose runtime issues")]
    pub async fn get_containerd_logs(
        &self,
        Parameters(GetContainerdLogsParams { path }): Parameters<GetContainerdLogsParams>,
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

    #[tool(description = "Reopen target container log")]
    pub async fn reopen_container_log(
        &self,
        Parameters(ReopenContainerLogParams { container_id }): Parameters<ReopenContainerLogParams>,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::container::reopen_container_log(&mut client_clone, container_id).await
            {
                Ok(_) => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "Container log reopened successfully",
                    )]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to reopen container log: {}",
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
        Parameters(CreatePodParams { name, namespace, uid, options }): Parameters<CreatePodParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(
            "Create pod request - name: {}, namespace: {}, uid: {}, options: {:?}",
            name, namespace, uid, options
        );
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::pod::create_pod(&mut client_clone, name, namespace, uid, options)
                .await
            {
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
        Parameters(RemovePodParams { pod_id }): Parameters<RemovePodParams>,
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
        Parameters(CreateContainerParams { pod_id, name, image, options, pod_config }): Parameters<CreateContainerParams>,
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
                pod_config,
            )
            .await
            {
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
        Parameters(RemoveContainerParams { container_id }): Parameters<RemoveContainerParams>,
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
        Parameters(StopPodParams { pod_id }): Parameters<StopPodParams>,
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
        Parameters(StartContainerParams { container_id }): Parameters<StartContainerParams>,
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
        Parameters(StopContainerParams { id, timeout }): Parameters<StopContainerParams>,
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
        Parameters(ExecSyncParams { container_id, command, timeout }): Parameters<ExecSyncParams>,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::container::exec_sync(
                &mut client_clone,
                container_id,
                command,
                timeout,
            )
            .await
            {
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
        Parameters(PullImageParams { image_reference }): Parameters<PullImageParams>,
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
        Parameters(RemoveImageParams { image_reference }): Parameters<RemoveImageParams>,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.image_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::image::remove_image(&mut client_clone, image_reference.clone()).await
            {
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
        Parameters(ContainerLogsParams { container_id, tail }): Parameters<ContainerLogsParams>,
    ) -> Result<CallToolResult, McpError> {
        let lock = self.runtime_client.lock().await;
        if let Some(client) = &*lock {
            let mut client_clone = client.clone();
            match crate::cri::container::container_logs(&mut client_clone, container_id).await {
                Ok((log_content, _log_path)) => {
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
        Parameters(ContainerStatsParams { container_id }): Parameters<ContainerStatsParams>,
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
        Parameters(PodStatsParams { pod_id }): Parameters<PodStatsParams>,
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

#[tool_handler]
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
            instructions: Some("This server provides tools to interact with Containerd through both CRI (Container Runtime Interface) and CTR (command line tool). CRI tools for K8s-style management: 'version', 'list_pods', 'list_containers', 'list_images', 'image_fs_info', 'create_pod', 'remove_pod', 'stop_pod', 'create_container', 'start_container', 'stop_container', 'remove_container', 'exec_sync', 'pull_image', 'remove_image', 'container_stats', 'pod_stats', 'container_logs'. CTR tools for direct containerd management (with _ctr suffix): 'run_ctr_command', 'list_containers_ctr', 'list_images_ctr', 'list_tasks_ctr', 'pull_image_ctr', 'remove_image_ctr', 'run_container_ctr', 'remove_container_ctr'. Use CRI tools for K8s-compatible container management and CTR tools for direct containerd operations.".to_string()),
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
