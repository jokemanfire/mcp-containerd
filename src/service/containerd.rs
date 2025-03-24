use crate::cri::runtime::RuntimeService;
use crate::cri::image::ImageService;
use crate::cri::*;
use rmcp::handler::server::ServerHandler;
use anyhow::Result;
use std::sync::Arc;

// 版本服务
pub struct VersionService {
    runtime: Arc<RuntimeService>,
}

impl VersionService {
    pub fn new(runtime: Arc<RuntimeService>) -> Self {
        Self { runtime }
    }
    
    pub async fn get_version(&self) -> Result<VersionResponse> {
        let request = VersionRequest {
            version: "v1".to_string(),
        };
        self.runtime.version(request).await
    }
}

impl ServerHandler for VersionService {}

// 运行时服务
pub struct ContainerdRuntimeService {
    runtime: Arc<RuntimeService>,
}

impl ContainerdRuntimeService {
    pub fn new(runtime: Arc<RuntimeService>) -> Self {
        Self { runtime }
    }
    
    // 添加Pod相关方法
    pub async fn run_pod_sandbox(&self, pod_config: PodSandboxConfig, runtime_handler: String) -> Result<String> {
        let request = RunPodSandboxRequest {
            config: Some(pod_config),
            runtime_handler,
        };
        let response = self.runtime.run_pod_sandbox(request).await?;
        Ok(response.pod_sandbox_id)
    }
    
    pub async fn stop_pod_sandbox(&self, pod_sandbox_id: String) -> Result<()> {
        let request = StopPodSandboxRequest {
            pod_sandbox_id,
        };
        self.runtime.stop_pod_sandbox(request).await?;
        Ok(())
    }
    
    pub async fn remove_pod_sandbox(&self, pod_sandbox_id: String) -> Result<()> {
        let request = RemovePodSandboxRequest {
            pod_sandbox_id,
        };
        self.runtime.remove_pod_sandbox(request).await?;
        Ok(())
    }
    
    pub async fn list_pod_sandbox(&self, filter: Option<PodSandboxFilter>) -> Result<Vec<PodSandbox>> {
        let request = ListPodSandboxRequest {
            filter,
        };
        let response = self.runtime.list_pod_sandbox(request).await?;
        Ok(response.items)
    }
    
    pub async fn pod_sandbox_status(&self, pod_sandbox_id: String, verbose: bool) -> Result<PodSandboxStatus> {
        let request = PodSandboxStatusRequest {
            pod_sandbox_id,
            verbose,
        };
        let response = self.runtime.pod_sandbox_status(request).await?;
        match response.status {
            Some(status) => Ok(status),
            None => anyhow::bail!("No pod sandbox status returned"),
        }
    }
    
    // 添加容器相关方法
    pub async fn create_container(&self, 
        pod_sandbox_id: String, 
        config: ContainerConfig, 
        sandbox_config: PodSandboxConfig
    ) -> Result<String> {
        let request = CreateContainerRequest {
            pod_sandbox_id,
            config: Some(config),
            sandbox_config: Some(sandbox_config),
        };
        let response = self.runtime.create_container(request).await?;
        Ok(response.container_id)
    }
    
    pub async fn start_container(&self, container_id: String) -> Result<()> {
        let request = StartContainerRequest {
            container_id,
        };
        self.runtime.start_container(request).await?;
        Ok(())
    }
    
    pub async fn stop_container(&self, container_id: String, timeout: i64) -> Result<()> {
        let request = StopContainerRequest {
            container_id,
            timeout,
        };
        self.runtime.stop_container(request).await?;
        Ok(())
    }
    
    pub async fn remove_container(&self, container_id: String) -> Result<()> {
        let request = RemoveContainerRequest {
            container_id,
        };
        self.runtime.remove_container(request).await?;
        Ok(())
    }
    
    pub async fn list_containers(&self, filter: Option<ContainerFilter>) -> Result<Vec<Container>> {
        let request = ListContainersRequest {
            filter,
        };
        let response = self.runtime.list_containers(request).await?;
        Ok(response.containers)
    }
    
    pub async fn container_status(&self, container_id: String, verbose: bool) -> Result<ContainerStatus> {
        let request = ContainerStatusRequest {
            container_id,
            verbose,
        };
        let response = self.runtime.container_status(request).await?;
        match response.status {
            Some(status) => Ok(status),
            None => anyhow::bail!("No container status returned"),
        }
    }
    
    pub async fn exec_sync(&self, container_id: String, cmd: Vec<String>, timeout: i64) -> Result<(i32, Vec<u8>, Vec<u8>)> {
        let request = ExecSyncRequest {
            container_id,
            cmd,
            timeout,
        };
        let response = self.runtime.exec_sync(request).await?;
        Ok((response.exit_code, response.stdout, response.stderr))
    }
}

impl ServerHandler for ContainerdRuntimeService {}

// 镜像服务
pub struct ContainerdImageService {
    image: Arc<ImageService>,
}

impl ContainerdImageService {
    pub fn new(image: Arc<ImageService>) -> Self {
        Self { image }
    }
    
    pub async fn list_images(&self, filter: Option<ImageFilter>) -> Result<Vec<Image>> {
        let request = ListImagesRequest {
            filter,
        };
        let response = self.image.list_images(request).await?;
        Ok(response.images)
    }
    
    pub async fn image_status(&self, image: ImageSpec, verbose: bool) -> Result<ImageStatus> {
        let request = ImageStatusRequest {
            image: Some(image),
            verbose,
        };
        let response = self.image.image_status(request).await?;
        match response.image {
            Some(status) => Ok(status),
            None => anyhow::bail!("No image status returned"),
        }
    }
    
    pub async fn pull_image(&self, image: ImageSpec, auth: Option<AuthConfig>, sandbox_config: Option<PodSandboxConfig>) -> Result<String> {
        let request = PullImageRequest {
            image: Some(image.clone()),
            auth,
            sandbox_config,
        };
        let response = self.image.pull_image(request).await?;
        Ok(response.image_ref)
    }
    
    pub async fn remove_image(&self, image: ImageSpec) -> Result<()> {
        let request = RemoveImageRequest {
            image: Some(image),
        };
        self.image.remove_image(request).await?;
        Ok(())
    }
    
    pub async fn image_fs_info(&self) -> Result<Vec<FilesystemUsage>> {
        let request = ImageFsInfoRequest {};
        let response = self.image.image_fs_info(request).await?;
        Ok(response.image_filesystems)
    }
}

impl ServerHandler for ContainerdImageService {} 