use crate::api::runtime::v1::{
    ContainerStatsRequest, CreateContainerRequest, RemoveContainerRequest,
};
use crate::cri::config::parse_container_config;
use anyhow::Result;
use tonic::transport::Channel;
use tracing::debug;

pub async fn create_container(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    pod_id: String,
    name: String,
    image: String,
    options: String,
    pod_config: String,
) -> Result<String, tonic::Status> {
    debug!(
        "Create container request - pod_id: {}, name: {}, image: {}, options: {:?}",
        pod_id, name, image, options
    );

    // Create a base config with required fields
    let mut container_config_value = serde_json::json!({
        "metadata": {
            "name": name
        },
        "image": {
            "image": image
        },
        "log_path": format!("{}/0.log", name)
    });

    // Merge the options
    let container_obj = container_config_value.as_object_mut().unwrap();
    let options_value = serde_json::from_str::<serde_json::Value>(&options).unwrap();
    for (key, value) in options_value.as_object().unwrap() {
        container_obj.insert(key.clone(), value.clone());
    }

    // Parse container configuration with defaults
    let container_config = parse_container_config(container_config_value);

    // Convert HashMap to JSON Value directly
    let pod_config_value = serde_json::from_str::<serde_json::Value>(&pod_config).unwrap();
    // Parse pod configuration for sandbox_config
    let sandbox_config = crate::cri::config::parse_pod_config(pod_config_value);

    let request = CreateContainerRequest {
        pod_sandbox_id: pod_id,
        config: Some(container_config),
        sandbox_config: Some(sandbox_config),
    };

    debug!("create container request: {:?}", request);

    let response = client.create_container(request).await?;
    Ok(response.into_inner().container_id)
}

pub async fn remove_container(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    container_id: String,
) -> Result<(), tonic::Status> {
    let request = RemoveContainerRequest { container_id };
    client.remove_container(request).await?;
    Ok(())
}

pub async fn start_container(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    container_id: String,
) -> Result<(), tonic::Status> {
    let request = crate::api::runtime::v1::StartContainerRequest { container_id };
    client.start_container(request).await?;
    Ok(())
}

pub async fn stop_container(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    container_id: String,
    timeout: i64,
) -> Result<(), tonic::Status> {
    let request = crate::api::runtime::v1::StopContainerRequest {
        container_id,
        timeout,
    };
    client.stop_container(request).await?;
    Ok(())
}

pub async fn container_stats(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    container_id: String,
) -> Result<crate::api::runtime::v1::ContainerStatsResponse, tonic::Status> {
    let request = ContainerStatsRequest { container_id };
    let response = client.container_stats(request).await?;
    Ok(response.into_inner())
}

pub async fn container_logs(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    container_id: String,
) -> Result<(String, String), tonic::Status> {
    let request = crate::api::runtime::v1::ContainerStatusRequest {
        container_id: container_id.clone(),
        verbose: true,
    };

    let status = client.container_status(request).await?;
    let status = status.into_inner();

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
            return Err(tonic::Status::not_found("Container status not available"));
        }
    };

    // Read the log file
    match std::fs::read_to_string(&log_path) {
        Ok(log_content) => Ok((log_content, log_path)),
        Err(e) => Err(tonic::Status::internal(format!(
            "Failed to read container logs at {}: {}",
            log_path, e
        ))),
    }
}

pub async fn exec_sync(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    container_id: String,
    command: String,
    timeout: Option<i64>,
) -> Result<crate::api::runtime::v1::ExecSyncResponse, tonic::Status> {
    let request = crate::api::runtime::v1::ExecSyncRequest {
        container_id,
        cmd: vec![command],
        timeout: timeout.unwrap_or(10), // Default timeout of 10 seconds
    };

    let response = client.exec_sync(request).await?;
    Ok(response.into_inner())
}

pub async fn reopen_container_log(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    container_id: String,
) -> Result<(), tonic::Status> {
    let request = crate::api::runtime::v1::ReopenContainerLogRequest { container_id };
    client.reopen_container_log(request).await?;
    Ok(())
}
