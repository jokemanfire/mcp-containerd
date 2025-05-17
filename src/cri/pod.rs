use crate::api::runtime::v1::{
    PodSandboxConfig, RunPodSandboxRequest, RunPodSandboxResponse,
    RemovePodSandboxRequest, RemovePodSandboxResponse,
    ListPodSandboxRequest, ListPodSandboxResponse,
    StopPodSandboxRequest, ListPodSandboxStatsRequest
};
use crate::cri::config::parse_pod_config;
use anyhow::Result;
use tonic::transport::Channel;
use tracing::debug;

pub async fn create_pod(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    name: String,
    namespace: String,
    uid: String,
    options: String,
) -> Result<(String, PodSandboxConfig), tonic::Status> {
    debug!(
        "Create pod request - name: {}, namespace: {}, uid: {}, options: {:?}",
        name, namespace, uid, options
    );

    // Create a base config with required fields
    let mut pod_config_value = serde_json::json!({
        "metadata": {
            "name": name,
            "namespace": namespace,
            "uid": uid,
            "attempt": 0
        },
        "hostname": format!("{}-{}", name, namespace),
    });

    // Merge the options
    if let Some(pod_obj) = pod_config_value.as_object_mut() {
        let options_value = serde_json::from_str::<serde_json::Value>(&options).unwrap();
        for (key, value) in options_value.as_object().unwrap() {
            pod_obj.insert(key.clone(), value.clone());
        }
    }

    // Parse pod configuration with defaults
    let pod_config = parse_pod_config(pod_config_value);

    let request = RunPodSandboxRequest {
        config: Some(pod_config.clone()),
        runtime_handler: "".to_string(),
    };
    debug!("run pod sandbox request: {:?}", request);

    let response = client.run_pod_sandbox(request).await?;
    let pod_id = response.into_inner().pod_sandbox_id;
    
    Ok((pod_id, pod_config))
}

pub async fn remove_pod(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    pod_id: String,
) -> Result<(), tonic::Status> {
    let request = RemovePodSandboxRequest {
        pod_sandbox_id: pod_id,
    };

    client.remove_pod_sandbox(request).await?;
    Ok(())
}

pub async fn stop_pod(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    pod_id: String,
) -> Result<(), tonic::Status> {
    let request = StopPodSandboxRequest {
        pod_sandbox_id: pod_id,
    };

    client.stop_pod_sandbox(request).await?;
    Ok(())
}

pub async fn list_pods(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
) -> Result<ListPodSandboxResponse, tonic::Status> {
    let request = ListPodSandboxRequest { filter: None };
    let response = client.list_pod_sandbox(request).await?;
    Ok(response.into_inner())
}

pub async fn pod_stats(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    pod_id: Option<String>,
) -> Result<crate::api::runtime::v1::ListPodSandboxStatsResponse, tonic::Status> {
    // Create filter if pod_id is provided
    let filter = match pod_id {
        Some(id) => Some(crate::api::runtime::v1::PodSandboxStatsFilter {
            id,
            label_selector: std::collections::HashMap::new(),
        }),
        None => None,
    };

    let request = ListPodSandboxStatsRequest { filter };
    let response = client.list_pod_sandbox_stats(request).await?;
    Ok(response.into_inner())
} 