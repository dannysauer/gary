use core::declare_plugin;
use core::plugins::runtime::*;
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct CriRuntimePlugin {
    _cri_client: String,
    #[allow(dead_code)]
    runner: Runtime,
}

impl Default for CriRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl CriRuntimePlugin {
    pub fn new() -> Self {
        CriRuntimePlugin {
            _cri_client: String::from("cri"),
            runner: Runtime::new().unwrap(),
        }
    }
}

declare_plugin!(CriRuntimePlugin, CriRuntimePlugin::new);

impl RuntimePlugin for CriRuntimePlugin {
    fn name(&self) -> String {
        "cri".to_string()
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

    fn start_workload(&mut self, _id: String) -> Option<RuntimeError> {
        Some(RuntimeError::new(RuntimeErrorType::Unimplemented))
    }

    fn stop_workload(&self, _id: String, _timeout: i32) -> Option<RuntimeError> {
        Some(RuntimeError::new(RuntimeErrorType::Unimplemented))
    }

    fn remove_workload(&mut self, _id: String) -> Option<RuntimeError> {
        Some(RuntimeError::new(RuntimeErrorType::Unimplemented))
    }

    fn status_workload(&mut self, _id: String) -> Result<WorkloadStatus, RuntimeError> {
        Err(RuntimeError::new(RuntimeErrorType::Unimplemented))
    }

    fn update_workload_resources(
        &self,
        _id: String,
        _rez: WorkloadResources,
    ) -> Option<RuntimeError> {
        Some(RuntimeError::new(RuntimeErrorType::Unimplemented))
    }

    fn exec_sync(
        &self,
        _id: String,
        _cmd: &[String],
        _timeout: i32,
    ) -> (&[u8], &[u8], Option<RuntimeError>) {
        (
            &[],
            &[],
            Some(RuntimeError::new(RuntimeErrorType::Unimplemented)),
        )
    }
}
