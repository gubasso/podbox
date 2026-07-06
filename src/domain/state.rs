use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SandboxState {
    Absent,
    Composed,
    ImageReady,
    Running,
    Drifted,
    Stopped,
    Failed,
    Unknown,
}

impl SandboxState {
    pub(crate) fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (SandboxState::Absent, SandboxState::Composed)
                | (SandboxState::Composed, SandboxState::ImageReady)
                | (SandboxState::ImageReady, SandboxState::Running)
                | (
                    SandboxState::Running,
                    SandboxState::Drifted | SandboxState::Stopped | SandboxState::Failed
                )
                | (
                    SandboxState::Drifted,
                    SandboxState::Composed | SandboxState::Stopped | SandboxState::Failed
                )
                | (
                    SandboxState::Stopped,
                    SandboxState::Running | SandboxState::Composed | SandboxState::Absent
                )
                | (
                    SandboxState::Failed,
                    SandboxState::Composed | SandboxState::Absent
                )
                | (
                    SandboxState::Unknown,
                    SandboxState::Composed | SandboxState::Absent
                )
        )
    }
}
