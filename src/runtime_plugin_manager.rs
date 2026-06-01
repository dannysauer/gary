use core::plugins::runtime::*;

use libloading::{Library, Symbol};
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct RuntimePluginManager {
    plugins: Vec<Box<dyn RuntimePlugin>>,
    loaded_libraries: Vec<Library>,
}

#[allow(dead_code)]
impl RuntimePluginManager {
    pub fn new() -> RuntimePluginManager {
        RuntimePluginManager {
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
        }
    }

    pub fn load_plugins_in_dir(&mut self, dir: String) -> Result<(), ()> {
        let path = PathBuf::from(dir);
        if let Ok(files) = fs::read_dir(path) {
            for f in files.flatten() {
                unsafe {
                    let _ = self.load_plugin(f.path());
                }
            }
        }
        Ok(())
    }

    pub fn load_in_memory_plugin(&mut self, plugin: Box<dyn RuntimePlugin>) {
        self.plugins.push(plugin);
    }

    pub unsafe fn load_plugin(&mut self, file: PathBuf) -> Result<(), ()> {
        type PluginCreate = unsafe fn() -> *mut dyn RuntimePlugin;

        let lib = Library::new(file.into_os_string()).unwrap();
        self.loaded_libraries.push(lib);
        let lib = self.loaded_libraries.last().unwrap();

        let constructor: Symbol<PluginCreate> = lib.get(b"_plugin_create").unwrap();
        let boxed_raw = constructor();
        let plugin: Box<dyn RuntimePlugin> = Box::from_raw(boxed_raw);
        plugin.on_plugin_load();
        self.plugins.push(plugin);
        Ok(())
    }

    pub fn get_plugin_name(&self, id: String) -> Result<String, RuntimeError> {
        for plugin in &self.plugins {
            if plugin.name() == id {
                return Ok(plugin.name());
            }
        }
        Err(RuntimeError::new(RuntimeErrorType::Unknown))
    }

    pub fn create_workload_by_name(
        &self,
        plugin_name: String,
        sandbox_id: String,
        config: &RuntimeConfig,
        options: &Option<SandboxConfig>,
    ) -> Result<String, RuntimeError> {
        for plugin in &self.plugins {
            if plugin.name() == plugin_name {
                return plugin.create_workload(sandbox_id, config, options);
            }
        }
        Err(RuntimeError::new(RuntimeErrorType::Unknown))
    }

    pub fn create_workload_by_type(
        &self,
        plugin_type: RuntimeFeatures,
        sandbox_id: String,
        config: &RuntimeConfig,
        options: &Option<SandboxConfig>,
    ) -> Result<(String, String), RuntimeError> {
        for plugin in &self.plugins {
            for feature in plugin.get_features() {
                if feature == plugin_type {
                    return match plugin.create_workload(sandbox_id, config, options) {
                        Ok(r) => Ok((plugin.name(), r)),
                        _ => Err(RuntimeError::new(RuntimeErrorType::Unknown)),
                    };
                }
            }
        }
        Err(RuntimeError::new(RuntimeErrorType::Unknown))
    }

    pub fn start_workload(
        &mut self,
        workload_id: String,
        plugin_name: String,
    ) -> Option<RuntimeError> {
        for plugin in &mut self.plugins {
            if plugin.name() == plugin_name {
                return plugin.start_workload(workload_id);
            }
        }
        Some(RuntimeError::new(RuntimeErrorType::Unknown))
    }

    pub fn stop_workload(
        &self,
        workload_id: String,
        plugin_name: String,
        timeout: i32,
    ) -> Option<RuntimeError> {
        for plugin in &self.plugins {
            if plugin.name() == plugin_name {
                return plugin.stop_workload(workload_id, timeout);
            }
        }
        Some(RuntimeError::new(RuntimeErrorType::Unknown))
    }

    pub fn remove_workload(
        &mut self,
        workload_id: String,
        plugin_name: String,
    ) -> Option<RuntimeError> {
        for plugin in &mut self.plugins {
            if plugin.name() == plugin_name {
                return plugin.remove_workload(workload_id);
            }
        }
        Some(RuntimeError::new(RuntimeErrorType::Unknown))
    }

    pub fn status_workload(
        &mut self,
        workload_id: String,
        plugin_name: String,
    ) -> Result<WorkloadStatus, RuntimeError> {
        for plugin in &mut self.plugins {
            if plugin.name() == plugin_name {
                return plugin.status_workload(workload_id);
            }
        }
        Err(RuntimeError::new(RuntimeErrorType::Unknown))
    }
}
