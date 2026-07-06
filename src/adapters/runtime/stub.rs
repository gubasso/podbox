use std::sync::Mutex;

use crate::adapters::runtime::{
    BuildImageRequest, DevcontainerRunSpec, RuntimeAdapter, RuntimeError, RuntimeStatus,
};

#[derive(Debug)]
pub(crate) struct StubRuntimeAdapter {
    status: RuntimeStatus,
    pub(crate) builds: Mutex<Vec<BuildImageRequest>>,
}

impl StubRuntimeAdapter {
    pub(crate) fn exiting(code: u8) -> Self {
        Self {
            status: RuntimeStatus { code },
            builds: Mutex::new(Vec::new()),
        }
    }
}

impl RuntimeAdapter for StubRuntimeAdapter {
    fn build_image(&self, request: &BuildImageRequest) -> Result<RuntimeStatus, RuntimeError> {
        self.builds
            .lock()
            .expect("stub builds")
            .push(request.clone());
        Ok(self.status.clone())
    }

    fn run_devcontainer(&self, _spec: &DevcontainerRunSpec) -> Result<RuntimeStatus, RuntimeError> {
        Ok(self.status.clone())
    }
}
