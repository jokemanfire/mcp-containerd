use crate::cri::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

// 运行时服务MCP实现
pub struct RuntimeService {
    client: Arc<Mutex<Option<CriClient>>>,
}

impl RuntimeService {
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

    // 版本信息
    pub async fn version(&self, request: VersionRequest) -> Result<VersionResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().version(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 创建Pod
    pub async fn run_pod_sandbox(
        &self,
        request: RunPodSandboxRequest,
    ) -> Result<RunPodSandboxResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().run_pod_sandbox(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 停止Pod
    pub async fn stop_pod_sandbox(
        &self,
        request: StopPodSandboxRequest,
    ) -> Result<StopPodSandboxResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().stop_pod_sandbox(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 删除Pod
    pub async fn remove_pod_sandbox(
        &self,
        request: RemovePodSandboxRequest,
    ) -> Result<RemovePodSandboxResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().remove_pod_sandbox(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 获取Pod状态
    pub async fn pod_sandbox_status(
        &self,
        request: PodSandboxStatusRequest,
    ) -> Result<PodSandboxStatusResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().pod_sandbox_status(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 列出Pod
    pub async fn list_pod_sandbox(
        &self,
        request: ListPodSandboxRequest,
    ) -> Result<ListPodSandboxResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().list_pod_sandbox(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 创建容器
    pub async fn create_container(
        &self,
        request: CreateContainerRequest,
    ) -> Result<CreateContainerResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().create_container(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 启动容器
    pub async fn start_container(
        &self,
        request: StartContainerRequest,
    ) -> Result<StartContainerResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().start_container(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 停止容器
    pub async fn stop_container(
        &self,
        request: StopContainerRequest,
    ) -> Result<StopContainerResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().stop_container(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 删除容器
    pub async fn remove_container(
        &self,
        request: RemoveContainerRequest,
    ) -> Result<RemoveContainerResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().remove_container(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 列出容器
    pub async fn list_containers(
        &self,
        request: ListContainersRequest,
    ) -> Result<ListContainersResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().list_containers(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 容器状态
    pub async fn container_status(
        &self,
        request: ContainerStatusRequest,
    ) -> Result<ContainerStatusResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().container_status(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 更新容器资源
    pub async fn update_container_resources(
        &self,
        request: UpdateContainerResourcesRequest,
    ) -> Result<UpdateContainerResourcesResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().update_container_resources(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }

    // 执行命令
    pub async fn exec_sync(&self, request: ExecSyncRequest) -> Result<ExecSyncResponse> {
        let lock = self.client.lock().await;
        if let Some(client) = &*lock {
            if let Some(service) = client.runtime_service() {
                let response = service.clone().exec_sync(request).await?;
                return Ok(response.into_inner());
            }
        }
        anyhow::bail!("Runtime service not connected")
    }
}
