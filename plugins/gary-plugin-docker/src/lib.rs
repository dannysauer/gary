use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, StatsOptions};
use bollard::Docker;
use core::declare_plugin;
use core::plugins::runtime::*;
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct ContainerdRuntimePlugin {
    docker: Docker,
    runner: Runtime,
}

impl Default for ContainerdRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerdRuntimePlugin {
    pub fn new() -> Self {
        ContainerdRuntimePlugin {
            runner: Runtime::new().unwrap(),
            docker: Docker::connect_with_local_defaults().unwrap(),
        }
    }
}

declare_plugin!(ContainerdRuntimePlugin, ContainerdRuntimePlugin::new);

impl RuntimePlugin for ContainerdRuntimePlugin {
    fn name(&self) -> String {
        "docker".to_string()
    }

    fn on_plugin_load(&self) {}
    fn on_plugin_unload(&self) {}

    fn get_features(&self) -> Vec<RuntimeFeatures> {
        vec![RuntimeFeatures::WorkloadRunner, RuntimeFeatures::Container]
    }

    fn get_version(&self) -> i32 {
        0
    }

    fn create_workload(
        &self,
        _id: String,
        _config: &RuntimeConfig,
        _options: &Option<SandboxConfig>,
    ) -> Result<String, RuntimeError> {
        Err(RuntimeError::new(RuntimeErrorType::Unimplemented))
    }

    fn start_workload(&mut self, id: String) -> Option<RuntimeError> {
        let nginx_config = Config {
            image: Some(id.as_str()),
            env: Some(vec![]),
            ..Default::default()
        };
        let create_opts = CreateContainerOptions {
            name: id.as_str(),
            platform: None,
        };
        let create_result = self.runner.block_on(
            self.docker
                .create_container(Some(create_opts), nginx_config),
        );
        if create_result.is_err() {
            return Some(RuntimeError::new(RuntimeErrorType::Unknown));
        }
        let start_result = self.runner.block_on(
            self.docker
                .start_container(&id, None::<StartContainerOptions<String>>),
        );
        match start_result {
            Ok(_) => None,
            Err(_) => Some(RuntimeError::new(RuntimeErrorType::Unknown)),
        }
    }

    fn stop_workload(&self, _id: String, _timeout: i32) -> Option<RuntimeError> {
        None
    }

    fn remove_workload(&mut self, _id: String) -> Option<RuntimeError> {
        None
    }

    fn status_workload(&mut self, id: String) -> Result<WorkloadStatus, RuntimeError> {
        use futures_util::StreamExt;

        let mut stream = self.docker.stats(
            &id,
            Some(StatsOptions {
                stream: false,
                ..Default::default()
            }),
        );
        let stat = self.runner.block_on(stream.next());
        match stat {
            Some(Ok(s)) => Ok(WorkloadStatus {
                current_memory: s.memory_stats.usage.unwrap_or(0),
                max_memory: s.memory_stats.max_usage.unwrap_or(0),
                workload_status: CurrentWorkloadStatus::Running,
            }),
            _ => Err(RuntimeError::new(RuntimeErrorType::Unknown)),
        }
    }

    fn update_workload_resources(
        &self,
        _id: String,
        _rez: WorkloadResources,
    ) -> Option<RuntimeError> {
        None
    }

    fn exec_sync(
        &self,
        _id: String,
        _cmd: &[String],
        _timeout: i32,
    ) -> (&[u8], &[u8], Option<RuntimeError>) {
        (&[], &[], None)
    }
}
