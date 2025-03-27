pub mod image;
pub mod runtime;

// 导入生成的protobuf代码
pub use crate::api::runtime::v1::*;
pub use crate::api::runtime::v1::{ImageServiceClient, RuntimeServiceClient};

// CRI服务客户端包装器
pub struct CriClient {
    runtime_service: Option<RuntimeServiceClient<tonic::transport::Channel>>,
    image_service: Option<ImageServiceClient<tonic::transport::Channel>>,
}

impl CriClient {
    pub async fn connect(endpoint: &str) -> anyhow::Result<Self> {
        let channel = tonic::transport::Channel::from_shared(endpoint.to_string())?
            .connect()
            .await?;

        let runtime_service = Some(RuntimeServiceClient::new(channel.clone()));
        let image_service = Some(ImageServiceClient::new(channel));

        Ok(Self {
            runtime_service,
            image_service,
        })
    }

    pub fn runtime_service(&self) -> Option<&RuntimeServiceClient<tonic::transport::Channel>> {
        self.runtime_service.as_ref()
    }

    pub fn image_service(&self) -> Option<&ImageServiceClient<tonic::transport::Channel>> {
        self.image_service.as_ref()
    }
}
