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

/// Helper trait for extracting and converting values from JSON maps
trait ValueExtractor {
    fn extract_string(&self, key: &str) -> Option<String>;
    fn extract_u32(&self, key: &str) -> Option<u32>;
    fn extract_bool(&self, key: &str) -> Option<bool>;
    fn extract_map(&self, key: &str) -> Option<Map<String, Value>>;
    fn extract_value(&self, key: &str) -> Option<&Value>;
}

impl ValueExtractor for Map<String, Value> {
    fn extract_string(&self, key: &str) -> Option<String> {
        self.get(key)?.as_str().map(|s| s.to_string())
    }

    fn extract_u32(&self, key: &str) -> Option<u32> {
        self.get(key)?.as_u64().map(|n| n as u32)
    }

    fn extract_bool(&self, key: &str) -> Option<bool> {
        self.get(key)?.as_bool()
    }

    fn extract_map(&self, key: &str) -> Option<Map<String, Value>> {
        serde_json::from_value(self.get(key)?.clone()).ok()
    }

    fn extract_value(&self, key: &str) -> Option<&Value> {
        self.get(key)
    }
}

/// Helper macro for updating fields conditionally
macro_rules! update_if_some {
    ($target:expr, $source:expr) => {
        if let Some(value) = $source {
            $target = value;
        }
    };
}

/// Helper macro for extending collections conditionally
macro_rules! extend_if_some {
    ($target:expr, $source:expr) => {
        if let Some(value) = $source {
            $target.extend(value);
        }
    };
}

/// Helper function to parse typed values from JSON
fn parse_typed_field<T>(map: &Map<String, Value>, key: &str) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    map.extract_value(key)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}

/// Creates default pod sandbox metadata
fn default_pod_metadata() -> PodSandboxMetadata {
    PodSandboxMetadata {
        name: "default-pod".to_string(),
        uid: Uuid::new_v4().to_string(),
        namespace: "default".to_string(),
        attempt: 0,
    }
}

/// Creates default pod sandbox configuration
fn default_pod_config() -> PodSandboxConfig {
    PodSandboxConfig {
        metadata: Some(default_pod_metadata()),
        hostname: "default-hostname".to_string(),
        log_directory: "/var/log/pods".to_string(),
        dns_config: None,
        port_mappings: vec![],
        labels: HashMap::new(),
        annotations: HashMap::new(),
        linux: None,
        windows: None,
    }
}

/// Parse pod metadata from JSON map
fn parse_pod_metadata(metadata_map: &Map<String, Value>) -> PodSandboxMetadata {
    let mut metadata = default_pod_metadata();
    
    update_if_some!(metadata.name, metadata_map.extract_string("name"));
    update_if_some!(metadata.uid, metadata_map.extract_string("uid"));
    update_if_some!(metadata.namespace, metadata_map.extract_string("namespace"));
    update_if_some!(metadata.attempt, metadata_map.extract_u32("attempt"));
    
    metadata
}

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

    let mut pod_config = default_pod_config();

    // Try to parse as JSON map first
    if let Ok(user_config) = serde_json::from_value::<Map<String, Value>>(config.clone()) {
        parse_pod_config_from_map(&mut pod_config, &user_config);
    } else if let Ok(direct_config) = serde_json::from_value::<PodSandboxConfig>(config) {
        // Fallback to direct parsing
        merge_pod_config(&mut pod_config, direct_config);
    }

    pod_config
}

/// Parse pod configuration from a JSON map
fn parse_pod_config_from_map(pod_config: &mut PodSandboxConfig, user_config: &Map<String, Value>) {
    // Handle metadata
    if let Some(metadata_map) = user_config.extract_map("metadata") {
        pod_config.metadata = Some(parse_pod_metadata(&metadata_map));
    }

    // Handle basic string fields
    update_if_some!(pod_config.hostname, user_config.extract_string("hostname"));
    update_if_some!(pod_config.log_directory, user_config.extract_string("log_directory"));

    // Handle complex typed fields
    if let Some(dns_config) = parse_typed_field::<DnsConfig>(user_config, "dns_config") {
        pod_config.dns_config = Some(dns_config);
    }

    if let Some(port_mappings) = parse_typed_field::<Vec<PortMapping>>(user_config, "port_mappings") {
        pod_config.port_mappings = port_mappings;
    }

    if let Some(linux_config) = parse_typed_field::<LinuxPodSandboxConfig>(user_config, "linux") {
        pod_config.linux = Some(linux_config);
    }

    if let Some(windows_config) = parse_typed_field::<WindowsPodSandboxConfig>(user_config, "windows") {
        pod_config.windows = Some(windows_config);
    }

    // Handle labels and annotations
    extend_if_some!(pod_config.labels, parse_typed_field::<HashMap<String, String>>(user_config, "labels"));
    extend_if_some!(pod_config.annotations, parse_typed_field::<HashMap<String, String>>(user_config, "annotations"));
}

/// Merge a direct PodSandboxConfig into the default configuration
fn merge_pod_config(target: &mut PodSandboxConfig, source: PodSandboxConfig) {
    if source.metadata.is_some() {
        target.metadata = source.metadata;
    }

    if !source.hostname.is_empty() {
        target.hostname = source.hostname;
    }

    if !source.log_directory.is_empty() {
        target.log_directory = source.log_directory;
    }

    if source.dns_config.is_some() {
        target.dns_config = source.dns_config;
    }

    if !source.port_mappings.is_empty() {
        target.port_mappings = source.port_mappings;
    }

    if !source.labels.is_empty() {
        target.labels = source.labels;
    }

    if !source.annotations.is_empty() {
        target.annotations = source.annotations;
    }

    if source.linux.is_some() {
        target.linux = source.linux;
    }

    if source.windows.is_some() {
        target.windows = source.windows;
    }
}

/// Creates default container metadata
fn default_container_metadata() -> ContainerMetadata {
    ContainerMetadata {
        name: format!("container-{}", &Uuid::new_v4().to_string()[..8]),
        attempt: 0,
    }
}

/// Creates default image specification
fn default_image_spec() -> ImageSpec {
    ImageSpec {
        image: "nginx:latest".to_string(),
        annotations: HashMap::new(),
        runtime_handler: "".to_string(),
        user_specified_image: "".to_string(),
    }
}

/// Creates default container configuration
fn default_container_config() -> ContainerConfig {
    ContainerConfig {
        metadata: Some(default_container_metadata()),
        image: Some(default_image_spec()),
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
    }
}

/// Parse container metadata from JSON map
fn parse_container_metadata(metadata_map: &Map<String, Value>) -> ContainerMetadata {
    let mut metadata = default_container_metadata();
    
    update_if_some!(metadata.name, metadata_map.extract_string("name"));
    update_if_some!(metadata.attempt, metadata_map.extract_u32("attempt"));
    
    metadata
}

/// Parse image specification from JSON map
fn parse_image_spec(image_map: &Map<String, Value>) -> ImageSpec {
    let mut image_spec = default_image_spec();
    
    update_if_some!(image_spec.image, image_map.extract_string("image"));
    update_if_some!(image_spec.runtime_handler, image_map.extract_string("runtime_handler"));
    update_if_some!(image_spec.user_specified_image, image_map.extract_string("user_specified_image"));
    
    if let Some(annotations) = parse_typed_field::<HashMap<String, String>>(image_map, "annotations") {
        image_spec.annotations = annotations;
    }
    
    image_spec
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

    let mut container_config = default_container_config();

    // Try to parse as JSON map first
    if let Ok(user_config) = serde_json::from_value::<Map<String, Value>>(config.clone()) {
        parse_container_config_from_map(&mut container_config, &user_config);
    } else if let Ok(direct_config) = serde_json::from_value::<ContainerConfig>(config) {
        // Fallback to direct parsing
        merge_container_config(&mut container_config, direct_config);
    }

    container_config
}

/// Parse container configuration from a JSON map
fn parse_container_config_from_map(container_config: &mut ContainerConfig, user_config: &Map<String, Value>) {
    // Handle metadata
    if let Some(metadata_map) = user_config.extract_map("metadata") {
        container_config.metadata = Some(parse_container_metadata(&metadata_map));
    }

    // Handle image configuration
    if let Some(image_map) = user_config.extract_map("image") {
        container_config.image = Some(parse_image_spec(&image_map));
    } else if let Some(image_ref) = user_config.extract_string("image") {
        // Handle simple image reference as string
        if let Some(ref mut image_spec) = container_config.image {
            image_spec.image = image_ref;
        }
    }

    // Handle basic string fields
    update_if_some!(container_config.working_dir, user_config.extract_string("working_dir"));
    update_if_some!(container_config.log_path, user_config.extract_string("log_path"));

    // Handle boolean fields
    update_if_some!(container_config.stdin, user_config.extract_bool("stdin"));
    update_if_some!(container_config.stdin_once, user_config.extract_bool("stdin_once"));
    update_if_some!(container_config.tty, user_config.extract_bool("tty"));

    // Handle vector fields
    if let Some(command) = parse_typed_field::<Vec<String>>(user_config, "command") {
        container_config.command = command;
    }

    if let Some(args) = parse_typed_field::<Vec<String>>(user_config, "args") {
        container_config.args = args;
    }

    if let Some(envs) = parse_typed_field::<Vec<KeyValue>>(user_config, "envs") {
        container_config.envs = envs;
    }

    if let Some(mounts) = parse_typed_field::<Vec<Mount>>(user_config, "mounts") {
        container_config.mounts = mounts;
    }

    if let Some(devices) = parse_typed_field::<Vec<Device>>(user_config, "devices") {
        container_config.devices = devices;
    }

    if let Some(cdi_devices) = parse_typed_field::<Vec<CdiDevice>>(user_config, "cdi_devices") {
        container_config.cdi_devices = cdi_devices;
    }

    // Handle complex typed fields
    if let Some(linux_config) = parse_typed_field::<LinuxContainerConfig>(user_config, "linux") {
        container_config.linux = Some(linux_config);
    }

    if let Some(windows_config) = parse_typed_field::<WindowsContainerConfig>(user_config, "windows") {
        container_config.windows = Some(windows_config);
    }

    // Handle labels and annotations
    extend_if_some!(container_config.labels, parse_typed_field::<HashMap<String, String>>(user_config, "labels"));
    extend_if_some!(container_config.annotations, parse_typed_field::<HashMap<String, String>>(user_config, "annotations"));
}

/// Merge a direct ContainerConfig into the default configuration
fn merge_container_config(target: &mut ContainerConfig, source: ContainerConfig) {
    if source.metadata.is_some() {
        target.metadata = source.metadata;
    }

    if source.image.is_some() {
        target.image = source.image;
    }

    if !source.command.is_empty() {
        target.command = source.command;
    }

    if !source.args.is_empty() {
        target.args = source.args;
    }

    if !source.working_dir.is_empty() {
        target.working_dir = source.working_dir;
    }

    if !source.envs.is_empty() {
        target.envs = source.envs;
    }

    if !source.mounts.is_empty() {
        target.mounts = source.mounts;
    }

    if !source.devices.is_empty() {
        target.devices = source.devices;
    }

    if !source.labels.is_empty() {
        target.labels = source.labels;
    }

    if !source.annotations.is_empty() {
        target.annotations = source.annotations;
    }

    if !source.log_path.is_empty() {
        target.log_path = source.log_path;
    }

    target.stdin = source.stdin;
    target.stdin_once = source.stdin_once;
    target.tty = source.tty;

    if source.linux.is_some() {
        target.linux = source.linux;
    }

    if source.windows.is_some() {
        target.windows = source.windows;
    }

    if !source.cdi_devices.is_empty() {
        target.cdi_devices = source.cdi_devices;
    }
}
