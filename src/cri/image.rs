use crate::cri::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

// 镜像服务MCP实现
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
    
    // 列出镜像
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
    
    // 镜像状态
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
    
    // 拉取镜像
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
    
    // 删除镜像
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
    
    // 镜像文件系统信息
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