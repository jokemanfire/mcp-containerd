use anyhow::Result;
use std::process::{Command, Output};

// CtrCmd provides functionality to execute containerd cli commands
pub struct CtrCmd {
    // Path to the ctr binary
    binary: String,
    // Namespace for containerd operations
    namespace: String,
    // Address of the containerd socket
    address: String,
}

impl CtrCmd {
    // Create a new CtrCmd instance with default settings
    pub fn new() -> Self {
        Self {
            binary: "ctr".to_string(),
            namespace: "default".to_string(),
            address: "/run/containerd/containerd.sock".to_string(),
        }
    }

    // Create a new CtrCmd with custom settings
    pub fn with_config(binary: String, namespace: String, address: String) -> Self {
        Self {
            binary,
            namespace,
            address,
        }
    }

    // Execute a ctr command with the given arguments
    pub fn execute(&self, args: Vec<String>) -> Result<Output> {
        let mut cmd = Command::new(&self.binary);
        
        // Add the namespace and address flags
        cmd.arg("--namespace")
           .arg(&self.namespace)
           .arg("--address")
           .arg(&self.address);
        
        // Add the command arguments
        cmd.args(args);
        
        // Execute the command and return the result
        Ok(cmd.output()?)
    }

    // Execute a container list command
    pub fn containers_list(&self) -> Result<Output> {
        self.execute(vec!["container".to_string(), "list".to_string()])
    }

    // Execute an image list command
    pub fn images_list(&self) -> Result<Output> {
        self.execute(vec!["image".to_string(), "list".to_string()])
    }

    // Execute a task list command
    pub fn tasks_list(&self) -> Result<Output> {
        self.execute(vec!["task".to_string(), "list".to_string()])
    }

    // Pull an image from a registry
    pub fn image_pull(&self, image_ref: &str) -> Result<Output> {
        self.execute(vec!["image".to_string(), "pull".to_string(), image_ref.to_string()])
    }

    // Remove an image
    pub fn image_remove(&self, image_ref: &str) -> Result<Output> {
        self.execute(vec!["image".to_string(), "remove".to_string(), image_ref.to_string()])
    }

    // Run a container
    pub fn container_run(&self, image_ref: &str, id: &str, args: Vec<String>) -> Result<Output> {
        let mut cmd_args = vec!["container".to_string(), "run".to_string(), image_ref.to_string(), id.to_string()];
        cmd_args.extend(args);
        self.execute(cmd_args)
    }

    // Remove a container
    pub fn container_remove(&self, id: &str) -> Result<Output> {
        self.execute(vec!["container".to_string(), "remove".to_string(), id.to_string()])
    }

    // Custom command execution with formatted args
    pub fn custom_command(&self, command: &str, args: Vec<&str>) -> Result<Output> {
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        self.execute(vec![command.to_string()].into_iter().chain(args).collect())
    }
} 