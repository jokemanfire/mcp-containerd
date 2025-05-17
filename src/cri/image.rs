use crate::api::runtime::v1::{
    ImageFsInfoRequest, ImageFsInfoResponse, ImageSpec, ListImagesRequest, ListImagesResponse,
    PullImageRequest, RemoveImageRequest,
};
use anyhow::Result;
use std::collections::HashMap;
use tonic::transport::Channel;

pub async fn pull_image(
    client: &mut crate::api::runtime::v1::ImageServiceClient<Channel>,
    image_reference: String,
) -> Result<String, tonic::Status> {
    let request = PullImageRequest {
        image: Some(ImageSpec {
            image: image_reference,
            annotations: HashMap::new(),
            runtime_handler: "".to_string(),
            user_specified_image: "".to_string(),
        }),
        auth: None,
        sandbox_config: None,
    };

    let response = client.pull_image(request).await?;
    Ok(response.into_inner().image_ref)
}

pub async fn remove_image(
    client: &mut crate::api::runtime::v1::ImageServiceClient<Channel>,
    image_reference: String,
) -> Result<(), tonic::Status> {
    let request = RemoveImageRequest {
        image: Some(ImageSpec {
            image: image_reference,
            annotations: HashMap::new(),
            runtime_handler: "".to_string(),
            user_specified_image: "".to_string(),
        }),
    };

    client.remove_image(request).await?;
    Ok(())
}

pub async fn list_images(
    client: &mut crate::api::runtime::v1::ImageServiceClient<Channel>,
) -> Result<ListImagesResponse, tonic::Status> {
    let request = ListImagesRequest { filter: None };
    let response = client.list_images(request).await?;
    Ok(response.into_inner())
}

pub async fn image_fs_info(
    client: &mut crate::api::runtime::v1::ImageServiceClient<Channel>,
) -> Result<ImageFsInfoResponse, tonic::Status> {
    let request = ImageFsInfoRequest {};
    let response = client.image_fs_info(request).await?;
    Ok(response.into_inner())
}
