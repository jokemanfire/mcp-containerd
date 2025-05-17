use crate::api::runtime::v1::{VersionRequest, VersionResponse};
use anyhow::Result;
use tonic::transport::Channel;

pub async fn version(
    client: &mut crate::api::runtime::v1::RuntimeServiceClient<Channel>,
) -> Result<VersionResponse, tonic::Status> {
    let request = VersionRequest {
        version: "v1".to_string(),
    };
    let response = client.version(request).await?;
    Ok(response.into_inner())
}

pub async fn connect_runtime(endpoint: &str) -> Result<(
    crate::api::runtime::v1::RuntimeServiceClient<Channel>,
    crate::api::runtime::v1::ImageServiceClient<Channel>
), anyhow::Error> {
    let socket_path = endpoint
        .strip_prefix("unix://")
        .expect("endpoint must start with unix://")
        .to_string();

    let channel = tonic::transport::Endpoint::try_from("http://[::]:50051")?
        .connect_with_connector(tower::service_fn(move |_: tonic::transport::Uri| {
            let socket_path = socket_path.to_string();
            async move { tokio::net::UnixStream::connect(socket_path).await }
        }))
        .await?;

    let runtime_client = crate::api::runtime::v1::RuntimeServiceClient::new(channel.clone());
    let image_client = crate::api::runtime::v1::ImageServiceClient::new(channel);

    Ok((runtime_client, image_client))
} 