/*
 * Containerd CTR Service Implementation
 *
 * This service provides tools to interact with Containerd using the ctr CLI tool.
 * It offers a more direct approach to many containerd operations.
 *
 * Current Supported Tool Interfaces:
 * - run_ctr_command: Run any ctr command
 * - list_containers: List all containers using ctr
 * - list_images: List all images using ctr
 * - list_tasks: List all tasks using ctr
 * - pull_image: Pull an image using ctr
 * - remove_image: Remove an image using ctr
 * - run_container: Run a container using ctr
 * - remove_container: Remove a container using ctr
 */

use rmcp::{
    const_string, model::*, schemars, service::RequestContext, tool, Error as McpError, RoleServer,
    ServerHandler,
};
use std::str;
use crate::service::containerd::Server;
use tracing::debug;

#[cfg(feature = "ctr")]
use crate::ctr::cmd::CtrCmd;



#[tool(tool_box)]
impl Server {

    /// Helper function to create a CtrCmd instance
    #[cfg(feature = "ctr")]
    fn create_ctr_cmd(&self) -> CtrCmd {
        CtrCmd::with_config(
            self.binary.clone(),
            self.namespace.clone(),
            self.address.clone(),
        )
    }

    #[tool(description = "Run any ctr command with custom arguments")]
    #[cfg(feature = "ctr")]
    pub async fn run_ctr_command(
        &self,
        #[tool(param)]
        #[schemars(description = "The ctr command to run, e.g. 'container list', 'image pull <image>'")]
        command: String,
    ) -> Result<CallToolResult, McpError> {
        debug!("Running ctr command: {}", command);
        
        // Split the command into parts
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Command cannot be empty",
            )]));
        }
        
        let ctr_cmd = self.create_ctr_cmd();
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
    #[cfg(feature = "ctr")]
    pub async fn list_containers(&self) -> Result<CallToolResult, McpError> {
        debug!("Listing containers with ctr");
        
        let ctr_cmd = self.create_ctr_cmd();
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
    #[cfg(feature = "ctr")]
    pub async fn list_images(&self) -> Result<CallToolResult, McpError> {
        debug!("Listing images with ctr");
        
        let ctr_cmd = self.create_ctr_cmd();
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
    #[cfg(feature = "ctr")]
    pub async fn list_tasks(&self) -> Result<CallToolResult, McpError> {
        debug!("Listing tasks with ctr");
        
        let ctr_cmd = self.create_ctr_cmd();
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
    #[cfg(feature = "ctr")]
    pub async fn pull_image(
        &self,
        #[tool(param)]
        #[schemars(description = "The image reference to pull, e.g. 'docker.io/library/nginx:latest'")]
        image_reference: String,
    ) -> Result<CallToolResult, McpError> {
        debug!("Pulling image with ctr: {}", image_reference);
        
        let ctr_cmd = self.create_ctr_cmd();
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
    #[cfg(feature = "ctr")]
    pub async fn remove_image(
        &self,
        #[tool(param)]
        #[schemars(description = "The image reference to remove, e.g. 'docker.io/library/nginx:latest'")]
        image_reference: String,
    ) -> Result<CallToolResult, McpError> {
        debug!("Removing image with ctr: {}", image_reference);
        
        let ctr_cmd = self.create_ctr_cmd();
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
    #[cfg(feature = "ctr")]
    pub async fn run_container(
        &self,
        #[tool(param)]
        #[schemars(description = "The image reference to use, e.g. 'docker.io/library/nginx:latest'")]
        image_reference: String,
        
        #[tool(param)]
        #[schemars(description = "The container ID or name")]
        container_id: String,
        
        #[tool(param)]
        #[schemars(description = "Additional arguments for the container run command (as a space-separated string)")]
        args: String,
    ) -> Result<CallToolResult, McpError> {
        debug!(
            "Running container with ctr - image: {}, id: {}, args: {}",
            image_reference, container_id, args
        );
        
        let args_vec: Vec<String> = args
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        
        let ctr_cmd = self.create_ctr_cmd();
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
    #[cfg(feature = "ctr")]
    pub async fn remove_container(
        &self,
        #[tool(param)]
        #[schemars(description = "The container ID or name to remove")]
        container_id: String,
    ) -> Result<CallToolResult, McpError> {
        debug!("Removing container with ctr: {}", container_id);
        
        let ctr_cmd = self.create_ctr_cmd();
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
}

