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
