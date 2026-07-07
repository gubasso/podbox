use std::sync::Mutex;

use crate::adapters::runtime::{
    BuildImageRequest, DevcontainerRunSpec, GuestExecRequest, NetworkPolicyRequest, RuntimeAdapter,
    RuntimeError, RuntimeStatus, WorkspaceRuntimeSpec,
};
use crate::domain::workspace::WorkspaceIdentity;

#[derive(Debug)]
pub(crate) struct StubRuntimeAdapter {
    status: RuntimeStatus,
    pub(crate) builds: Mutex<Vec<BuildImageRequest>>,
    pub(crate) starts: Mutex<Vec<WorkspaceRuntimeSpec>>,
    pub(crate) execs: Mutex<Vec<GuestExecRequest>>,
    pub(crate) shells: Mutex<Vec<GuestExecRequest>>,
    pub(crate) stops: Mutex<Vec<WorkspaceIdentity>>,
    pub(crate) removes: Mutex<Vec<WorkspaceIdentity>>,
    pub(crate) policies: Mutex<Vec<NetworkPolicyRequest>>,
}

impl StubRuntimeAdapter {
    pub(crate) fn exiting(code: u8) -> Self {
        Self {
            status: RuntimeStatus { code },
            builds: Mutex::new(Vec::new()),
            starts: Mutex::new(Vec::new()),
            execs: Mutex::new(Vec::new()),
            shells: Mutex::new(Vec::new()),
            stops: Mutex::new(Vec::new()),
            removes: Mutex::new(Vec::new()),
            policies: Mutex::new(Vec::new()),
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

    fn start_workspace(&self, spec: &WorkspaceRuntimeSpec) -> Result<RuntimeStatus, RuntimeError> {
        self.starts.lock().expect("stub starts").push(spec.clone());
        Ok(self.status.clone())
    }

    fn exec(&self, request: &GuestExecRequest) -> Result<RuntimeStatus, RuntimeError> {
        self.execs.lock().expect("stub execs").push(request.clone());
        Ok(self.status.clone())
    }

    fn shell(&self, request: &GuestExecRequest) -> Result<RuntimeStatus, RuntimeError> {
        self.shells
            .lock()
            .expect("stub shells")
            .push(request.clone());
        Ok(self.status.clone())
    }

    fn stop_workspace(&self, identity: &WorkspaceIdentity) -> Result<RuntimeStatus, RuntimeError> {
        self.stops
            .lock()
            .expect("stub stops")
            .push(identity.clone());
        Ok(self.status.clone())
    }

    fn remove_workspace(
        &self,
        identity: &WorkspaceIdentity,
    ) -> Result<RuntimeStatus, RuntimeError> {
        self.removes
            .lock()
            .expect("stub removes")
            .push(identity.clone());
        Ok(self.status.clone())
    }

    fn apply_network_policy(
        &self,
        request: &NetworkPolicyRequest,
    ) -> Result<RuntimeStatus, RuntimeError> {
        self.policies
            .lock()
            .expect("stub policies")
            .push(request.clone());
        Ok(self.status.clone())
    }
}
