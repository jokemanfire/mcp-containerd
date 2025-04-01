use crate::cri::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

// image service
pub struct ImageService {
    client: Arc<Mutex<Option<CriClient>>>,
}

impl ImageService {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(&self, endpoint: &str) -> Result<()> {
        let client = CriClient::connect(endpoint).await?;
        let mut lock = self.client.lock().await;
        *lock = Some(client);
        Ok(())
    }

    // list images
    pub async fn list_images(&self, request: ListImagesRequest) -> Result<ListImagesResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.image_service() {
                let response = service.clone().list_images(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Image service not connected")
    }

    // image status
    pub async fn image_status(&self, request: ImageStatusRequest) -> Result<ImageStatusResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.image_service() {
                let response = service.clone().image_status(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Image service not connected")
    }

    // pull image
    pub async fn pull_image(&self, request: PullImageRequest) -> Result<PullImageResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.image_service() {
                let response = service.clone().pull_image(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Image service not connected")
    }

    // remove image
    pub async fn remove_image(&self, request: RemoveImageRequest) -> Result<RemoveImageResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.image_service() {
                let response = service.clone().remove_image(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Image service not connected")
    }

    // image fs info
    pub async fn image_fs_info(&self, request: ImageFsInfoRequest) -> Result<ImageFsInfoResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.image_service() {
                let response = service.clone().image_fs_info(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Image service not connected")
    }
}
