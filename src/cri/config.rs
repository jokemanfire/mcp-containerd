/**
 * Container Runtime Interface (CRI) Configuration Parser
 *
 * This module provides parsing utilities for CRI configuration objects.
 * It handles conversion from JSON to structured configuration objects with sensible defaults,
 * making it easier to work with the Container Runtime Interface.
 *
 * Key features:
 * - Default configuration generation for both pods and containers
 * - Robust error handling for missing or invalid fields
 * - Support for incremental configuration (only specify what you need)
 * - Type-safe conversion from unstructured JSON to CRI data structures
 */
use crate::api::runtime::v1::{
    CdiDevice, ContainerConfig, ContainerMetadata, Device, DnsConfig, ImageSpec, KeyValue,
    LinuxContainerConfig, LinuxPodSandboxConfig, Mount, PodSandboxConfig, PodSandboxMetadata,
    PortMapping, WindowsContainerConfig, WindowsPodSandboxConfig,
};
use serde_json::{Map, Value};
use std::collections::HashMap;
use tracing::debug;
use uuid::Uuid;

/**
 * Parse pod sandbox configuration from JSON value
 *
 * Creates a default PodSandboxConfig and merges it with the provided JSON configuration.
 * This function handles partial configurations gracefully - any fields not specified in
 * the input will use sensible defaults. This makes it ideal for creating pods with minimal
 * configuration while ensuring all required fields have values.
 *
 * Default values include:
 * - A randomly generated UUID
 * - "default-pod" name
 * - "default" namespace
 * - "default-hostname" hostname
 * - "/var/log/pods" log directory
 *
 * # Arguments
 *
 * * `config` - JSON value containing pod configuration, which can be partial or complete
 *
 * # Returns
 *
 * * `PodSandboxConfig` - The merged pod configuration with all required fields populated
 *
 * # Example
 *
 * ```
 * let json = serde_json::json!({
 *     "metadata": {
 *         "name": "my-pod"
 *     }
 * });
 * let pod_config = parse_pod_config(json);
 * // Returns a complete PodSandboxConfig with "my-pod" as name and defaults for other fields
 * ```
 */
pub fn parse_pod_config(config: Value) -> PodSandboxConfig {
    debug!("Parsing pod configuration: {:?}", config);

    // Create default config
    let mut pod_config = PodSandboxConfig {
        metadata: Some(PodSandboxMetadata {
            name: "default-pod".to_string(),
            uid: Uuid::new_v4().to_string(),
            namespace: "default".to_string(),
            attempt: 0,
        }),
        hostname: "default-hostname".to_string(),
        log_directory: "/var/log/pods".to_string(),
        dns_config: None,
        port_mappings: vec![],
        labels: HashMap::new(),
        annotations: HashMap::new(),
        linux: None,
        windows: None,
    };

    // Parse and merge user configuration
    if let Ok(user_config) = serde_json::from_value::<Map<String, Value>>(config.clone()) {
        // Handle metadata
        if let Some(metadata_value) = user_config.get("metadata") {
            if let Ok(metadata_map) =
                serde_json::from_value::<Map<String, Value>>(metadata_value.clone())
            {
                let mut metadata = PodSandboxMetadata {
                    name: "default-pod".to_string(),
                    uid: Uuid::new_v4().to_string(),
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

        // Handle basic fields
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

        // Handle DNS config
        if let Some(dns_value) = user_config.get("dns_config") {
            if let Ok(dns_config) = serde_json::from_value::<DnsConfig>(dns_value.clone()) {
                pod_config.dns_config = Some(dns_config);
            }
        }

        // Handle port mappings
        if let Some(port_mappings) = user_config.get("port_mappings") {
            if let Ok(mappings) = serde_json::from_value::<Vec<PortMapping>>(port_mappings.clone())
            {
                pod_config.port_mappings = mappings;
            }
        }

        // Handle labels
        if let Some(labels) = user_config.get("labels") {
            if let Ok(label_map) = serde_json::from_value::<HashMap<String, String>>(labels.clone())
            {
                pod_config.labels.extend(label_map);
            }
        }

        // Handle annotations
        if let Some(annotations) = user_config.get("annotations") {
            if let Ok(anno_map) =
                serde_json::from_value::<HashMap<String, String>>(annotations.clone())
            {
                pod_config.annotations.extend(anno_map);
            }
        }

        // Handle Linux config
        if let Some(linux_value) = user_config.get("linux") {
            if let Ok(linux_config) =
                serde_json::from_value::<LinuxPodSandboxConfig>(linux_value.clone())
            {
                pod_config.linux = Some(linux_config);
            }
        }

        // Handle Windows config
        if let Some(windows_value) = user_config.get("windows") {
            if let Ok(windows_config) =
                serde_json::from_value::<WindowsPodSandboxConfig>(windows_value.clone())
            {
                pod_config.windows = Some(windows_config);
            }
        }
    } else {
        // If the config is not a map, try to parse it as PodSandboxConfig
        if let Ok(user_pod_config) = serde_json::from_value::<PodSandboxConfig>(config) {
            // Only merge non-empty fields
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

    pod_config
}

/**
 * Parse container configuration from JSON value
 *
 * Creates a default ContainerConfig and merges it with the provided JSON configuration.
 * This function handles partial container configurations gracefully - any fields not specified
 * in the input will use sensible defaults. This makes it easier to create containers with minimal
 * configuration while ensuring all required fields have values.
 *
 * Default values include:
 * - A uniquely generated container name using UUID
 * - "nginx:latest" as the default image
 * - "/" as the default working directory
 * - Standard configuration for stdin, stdout, and tty
 *
 * # Arguments
 *
 * * `config` - JSON value containing container configuration, which can be partial or complete
 *
 * # Returns
 *
 * * `ContainerConfig` - The merged container configuration with all required fields populated
 *
 * # Example
 *
 * ```
 * let json = serde_json::json!({
 *     "metadata": {
 *         "name": "my-container"
 *     },
 *     "image": {
 *         "image": "busybox:latest"
 *     }
 * });
 * let container_config = parse_container_config(json);
 * // Returns a complete ContainerConfig with "my-container" as name,
 * // "busybox:latest" as image, and defaults for other fields
 * ```
 */
pub fn parse_container_config(config: Value) -> ContainerConfig {
    debug!("Parsing container configuration: {:?}", config);

    // Create default config
    let mut container_config = ContainerConfig {
        metadata: Some(ContainerMetadata {
            name: format!("container-{}", Uuid::new_v4().to_string()[..8].to_string()),
            attempt: 0,
        }),
        image: Some(ImageSpec {
            image: "nginx:latest".to_string(),
            annotations: HashMap::new(),
            runtime_handler: "".to_string(),
            user_specified_image: "".to_string(),
        }),
        command: vec![],
        args: vec![],
        working_dir: "/".to_string(),
        envs: vec![],
        mounts: vec![],
        devices: vec![],
        labels: HashMap::new(),
        annotations: HashMap::new(),
        log_path: "container.log".to_string(),
        stdin: false,
        stdin_once: false,
        tty: false,
        linux: None,
        windows: None,
        cdi_devices: vec![],
    };

    // Parse and merge user configuration
    if let Ok(user_config) = serde_json::from_value::<Map<String, Value>>(config.clone()) {
        // Handle metadata
        if let Some(metadata_value) = user_config.get("metadata") {
            if let Ok(metadata_map) =
                serde_json::from_value::<Map<String, Value>>(metadata_value.clone())
            {
                let mut metadata = ContainerMetadata {
                    name: format!("container-{}", Uuid::new_v4().to_string()[..8].to_string()),
                    attempt: 0,
                };

                if let Some(name) = metadata_map.get("name") {
                    if let Some(name_str) = name.as_str() {
                        metadata.name = name_str.to_string();
                    }
                }

                if let Some(attempt) = metadata_map.get("attempt") {
                    if let Some(attempt_num) = attempt.as_u64() {
                        metadata.attempt = attempt_num as u32;
                    }
                }

                container_config.metadata = Some(metadata);
            }
        }

        // Handle image
        if let Some(image_value) = user_config.get("image") {
            if let Some(image_ref) = image_value.get("image") {
                container_config.image.as_mut().unwrap().image =
                    image_ref.as_str().unwrap().to_string();
            }
        }

        // Handle command and args
        if let Some(command) = user_config.get("command") {
            if let Ok(command_vec) = serde_json::from_value::<Vec<String>>(command.clone()) {
                container_config.command = command_vec;
            }
        }

        if let Some(args) = user_config.get("args") {
            if let Ok(args_vec) = serde_json::from_value::<Vec<String>>(args.clone()) {
                container_config.args = args_vec;
            }
        }

        // Handle working directory
        if let Some(working_dir) = user_config.get("working_dir") {
            if let Some(working_dir_str) = working_dir.as_str() {
                container_config.working_dir = working_dir_str.to_string();
            }
        }

        // Handle environment variables
        if let Some(envs) = user_config.get("envs") {
            if let Ok(envs_vec) = serde_json::from_value::<Vec<KeyValue>>(envs.clone()) {
                container_config.envs = envs_vec;
            }
        }

        // Handle mounts
        if let Some(mounts) = user_config.get("mounts") {
            if let Ok(mounts_vec) = serde_json::from_value::<Vec<Mount>>(mounts.clone()) {
                container_config.mounts = mounts_vec;
            }
        }

        // Handle devices
        if let Some(devices) = user_config.get("devices") {
            if let Ok(devices_vec) = serde_json::from_value::<Vec<Device>>(devices.clone()) {
                container_config.devices = devices_vec;
            }
        }

        // Handle labels
        if let Some(labels) = user_config.get("labels") {
            if let Ok(labels_map) =
                serde_json::from_value::<HashMap<String, String>>(labels.clone())
            {
                container_config.labels.extend(labels_map);
            }
        }

        // Handle annotations
        if let Some(annotations) = user_config.get("annotations") {
            if let Ok(annotations_map) =
                serde_json::from_value::<HashMap<String, String>>(annotations.clone())
            {
                container_config.annotations.extend(annotations_map);
            }
        }

        // Handle log path
        if let Some(log_path) = user_config.get("log_path") {
            if let Some(log_path_str) = log_path.as_str() {
                container_config.log_path = log_path_str.to_string();
            }
        }

        // Handle stdin, stdin_once, tty
        if let Some(stdin) = user_config.get("stdin") {
            if let Some(stdin_bool) = stdin.as_bool() {
                container_config.stdin = stdin_bool;
            }
        }

        if let Some(stdin_once) = user_config.get("stdin_once") {
            if let Some(stdin_once_bool) = stdin_once.as_bool() {
                container_config.stdin_once = stdin_once_bool;
            }
        }

        if let Some(tty) = user_config.get("tty") {
            if let Some(tty_bool) = tty.as_bool() {
                container_config.tty = tty_bool;
            }
        }

        // Handle Linux config
        if let Some(linux_value) = user_config.get("linux") {
            if let Ok(linux_config) =
                serde_json::from_value::<LinuxContainerConfig>(linux_value.clone())
            {
                container_config.linux = Some(linux_config);
            }
        }

        // Handle Windows config
        if let Some(windows_value) = user_config.get("windows") {
            if let Ok(windows_config) =
                serde_json::from_value::<WindowsContainerConfig>(windows_value.clone())
            {
                container_config.windows = Some(windows_config);
            }
        }

        // Handle CDI devices
        if let Some(cdi_devices) = user_config.get("cdi_devices") {
            if let Ok(cdi_devices_vec) =
                serde_json::from_value::<Vec<CdiDevice>>(cdi_devices.clone())
            {
                container_config.cdi_devices = cdi_devices_vec;
            }
        }
    } else {
        // If the config is not a map, try to parse it as ContainerConfig
        if let Ok(direct_config) = serde_json::from_value::<ContainerConfig>(config) {
            // Only merge non-empty fields
            if direct_config.metadata.is_some() {
                container_config.metadata = direct_config.metadata;
            }

            if direct_config.image.is_some() {
                container_config.image = direct_config.image;
            }

            if !direct_config.command.is_empty() {
                container_config.command = direct_config.command;
            }

            if !direct_config.args.is_empty() {
                container_config.args = direct_config.args;
            }

            if !direct_config.working_dir.is_empty() {
                container_config.working_dir = direct_config.working_dir;
            }

            if !direct_config.envs.is_empty() {
                container_config.envs = direct_config.envs;
            }

            if !direct_config.mounts.is_empty() {
                container_config.mounts = direct_config.mounts;
            }

            if !direct_config.devices.is_empty() {
                container_config.devices = direct_config.devices;
            }

            if !direct_config.labels.is_empty() {
                container_config.labels = direct_config.labels;
            }

            if !direct_config.annotations.is_empty() {
                container_config.annotations = direct_config.annotations;
            }

            if !direct_config.log_path.is_empty() {
                container_config.log_path = direct_config.log_path;
            }

            container_config.stdin = direct_config.stdin;
            container_config.stdin_once = direct_config.stdin_once;
            container_config.tty = direct_config.tty;

            if direct_config.linux.is_some() {
                container_config.linux = direct_config.linux;
            }

            if direct_config.windows.is_some() {
                container_config.windows = direct_config.windows;
            }

            if !direct_config.cdi_devices.is_empty() {
                container_config.cdi_devices = direct_config.cdi_devices;
            }
        }
    }

    container_config
}
