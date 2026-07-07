use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Write},
    sync::{Arc, Mutex},
};

use camino::Utf8PathBuf;
use fs4::{FileExt, TryLockError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::{
    digest::DigestInput,
    state::SandboxState,
    workspace::{WorkspaceIdentity, WorkspaceState},
};

#[derive(Debug, Error)]
pub(crate) enum StateStoreError {
    #[error("state operation already in progress for `{identity}`")]
    OperationInProgress { identity: String },
    #[error("state I/O error at {path}: {source}")]
    Io {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("state serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub(crate) trait StateStore: Send + Sync {
    fn read(&self, identity: &WorkspaceIdentity)
    -> Result<Option<WorkspaceState>, StateStoreError>;
    fn begin_mutation(
        &self,
        identity: &WorkspaceIdentity,
    ) -> Result<Box<dyn StateMutationGuard>, StateStoreError>;
    fn write(&self, state: &WorkspaceState) -> Result<(), StateStoreError>;
    fn write_locked(
        &self,
        state: &WorkspaceState,
        guard: &dyn StateMutationGuard,
    ) -> Result<(), StateStoreError>;
    fn mark_failed(
        &self,
        identity: &WorkspaceIdentity,
        reason: String,
    ) -> Result<(), StateStoreError>;
}

pub(crate) trait StateMutationGuard: Send {
    fn key(&self) -> &str;
}

#[derive(Debug)]
struct FsMutationGuard {
    key: String,
    _file: File,
}

impl StateMutationGuard for FsMutationGuard {
    fn key(&self) -> &str {
        &self.key
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FsStateStore {
    root: Utf8PathBuf,
}

impl FsStateStore {
    pub(crate) fn new(root: Utf8PathBuf) -> Self {
        Self { root }
    }

    fn path(&self, identity: &WorkspaceIdentity) -> Utf8PathBuf {
        self.root
            .join("workspaces")
            .join(safe_key(identity))
            .join("state.json")
    }

    fn lock_path(&self, identity: &WorkspaceIdentity) -> Utf8PathBuf {
        self.root
            .join("locks")
            .join(format!("{}.lock", safe_key(identity)))
    }

    fn acquire(&self, identity: &WorkspaceIdentity) -> Result<File, StateStoreError> {
        let path = self.lock_path(identity);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| StateStoreError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        // Reclaiming a stale lock by unlinking the lock file is unnecessary and
        // unsafe: `fs4`'s advisory lock is `flock(2)`, which the kernel releases
        // automatically when the holder's last fd closes — including on process
        // death. So a lock file left by a crashed owner carries no live lock, and
        // this `try_lock` simply succeeds on the *same* inode. Unlinking to
        // "reclaim" would instead race a concurrent acquirer that has already
        // re-created and locked the path, removing a live holder's file and
        // breaking mutual exclusion (two inodes, one path). We therefore never
        // unlink: open the (possibly pre-existing) file and take the advisory lock.
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|source| StateStoreError::Io {
                path: path.clone(),
                source,
            })?;
        FileExt::try_lock(&file).map_err(|source| match source {
            TryLockError::WouldBlock => StateStoreError::OperationInProgress {
                identity: identity.label.0.clone(),
            },
            TryLockError::Error(source) => StateStoreError::Io {
                path: path.clone(),
                source,
            },
        })?;
        // Overwrite the recorded owner PID with ours while we hold the advisory
        // lock, so it never races. The PID is informational (crash diagnostics);
        // correctness of mutual exclusion rests on the advisory lock alone.
        record_lock_owner(&file, &path)?;
        Ok(file)
    }

    fn write_inner(&self, state: &WorkspaceState) -> Result<(), StateStoreError> {
        let path = self.path(&state.identity);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| StateStoreError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let tmp = path.with_extension("json.tmp");
        let body = serde_json::to_vec_pretty(state)?;
        fs::write(&tmp, body).map_err(|source| StateStoreError::Io {
            path: tmp.clone(),
            source,
        })?;
        fs::rename(&tmp, &path).map_err(|source| StateStoreError::Io { path, source })
    }
}

impl StateStore for FsStateStore {
    fn read(
        &self,
        identity: &WorkspaceIdentity,
    ) -> Result<Option<WorkspaceState>, StateStoreError> {
        let path = self.path(identity);
        match fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw).map(Some).map_err(Into::into),
            Err(source) if source.kind() == ErrorKind::NotFound => Ok(None),
            Err(source) => Err(StateStoreError::Io { path, source }),
        }
    }

    fn begin_mutation(
        &self,
        identity: &WorkspaceIdentity,
    ) -> Result<Box<dyn StateMutationGuard>, StateStoreError> {
        Ok(Box::new(FsMutationGuard {
            key: safe_key(identity),
            _file: self.acquire(identity)?,
        }))
    }

    fn write(&self, state: &WorkspaceState) -> Result<(), StateStoreError> {
        let _lock = self.acquire(&state.identity)?;
        self.write_inner(state)
    }

    fn write_locked(
        &self,
        state: &WorkspaceState,
        guard: &dyn StateMutationGuard,
    ) -> Result<(), StateStoreError> {
        let key = safe_key(&state.identity);
        if guard.key() != key {
            return Err(StateStoreError::OperationInProgress {
                identity: state.identity.label.0.clone(),
            });
        }
        self.write_inner(state)
    }

    fn mark_failed(
        &self,
        identity: &WorkspaceIdentity,
        reason: String,
    ) -> Result<(), StateStoreError> {
        self.write(&WorkspaceState {
            identity: identity.clone(),
            state: SandboxState::Failed,
            failure: Some(reason),
            image_freshness: None,
            reconcile_fingerprint: None,
        })
    }
}

#[derive(Clone, Default)]
pub(crate) struct MemoryStateStore {
    states: Arc<Mutex<BTreeMap<String, WorkspaceState>>>,
}

#[derive(Debug)]
struct MemoryMutationGuard {
    key: String,
}

impl StateMutationGuard for MemoryMutationGuard {
    fn key(&self) -> &str {
        &self.key
    }
}

impl StateStore for MemoryStateStore {
    fn read(
        &self,
        identity: &WorkspaceIdentity,
    ) -> Result<Option<WorkspaceState>, StateStoreError> {
        Ok(self
            .states
            .lock()
            .expect("state mutex")
            .get(&safe_key(identity))
            .cloned())
    }

    fn begin_mutation(
        &self,
        identity: &WorkspaceIdentity,
    ) -> Result<Box<dyn StateMutationGuard>, StateStoreError> {
        Ok(Box::new(MemoryMutationGuard {
            key: safe_key(identity),
        }))
    }

    fn write(&self, state: &WorkspaceState) -> Result<(), StateStoreError> {
        self.states
            .lock()
            .expect("state mutex")
            .insert(safe_key(&state.identity), state.clone());
        Ok(())
    }

    fn write_locked(
        &self,
        state: &WorkspaceState,
        guard: &dyn StateMutationGuard,
    ) -> Result<(), StateStoreError> {
        if guard.key() != safe_key(&state.identity) {
            return Err(StateStoreError::OperationInProgress {
                identity: state.identity.label.0.clone(),
            });
        }
        self.write(state)
    }

    fn mark_failed(
        &self,
        identity: &WorkspaceIdentity,
        reason: String,
    ) -> Result<(), StateStoreError> {
        self.write(&WorkspaceState {
            identity: identity.clone(),
            state: SandboxState::Failed,
            failure: Some(reason),
            image_freshness: None,
            reconcile_fingerprint: None,
        })
    }
}

/// Write the current process id into the lock file (truncating any prior owner).
fn record_lock_owner(file: &File, path: &Utf8PathBuf) -> Result<(), StateStoreError> {
    let mut file = file;
    // A freshly opened O_RDWR handle is positioned at 0; set_len does not move it,
    // so this overwrites the prior owner (if any) from the start.
    let write = |file: &mut &File| -> std::io::Result<()> {
        file.set_len(0)?;
        file.write_all(format!("{}\n", std::process::id()).as_bytes())?;
        file.flush()
    };
    write(&mut file).map_err(|source| StateStoreError::Io {
        path: path.clone(),
        source,
    })
}

/// Derive a per-identity, collision-resistant filesystem key.
///
/// A lossy char-substitution scheme aliases distinct identities (e.g. `/a-b` vs
/// `/a/b`) to one key, collapsing their state files and locks and breaking
/// per-identity isolation. Hashing the length-prefixed `(path, label)` pair makes
/// collisions cryptographically improbable while staying filesystem-safe.
fn safe_key(identity: &WorkspaceIdentity) -> String {
    let digest = DigestInput::new()
        .part("workspace-path", identity.path.0.as_bytes())
        .part("workspace-label", identity.label.0.as_bytes())
        .finish();
    digest
        .as_str()
        .strip_prefix("sha256:")
        .unwrap_or(digest.as_str())
        .to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct _StateStoreSchemaAnchor;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::workspace::{WorkspaceLabel, WorkspacePath};
    use tempfile::TempDir;

    fn identity(path: &str, label: &str) -> WorkspaceIdentity {
        WorkspaceIdentity {
            path: WorkspacePath(path.to_string()),
            label: WorkspaceLabel(label.to_string()),
        }
    }

    fn store(temp: &TempDir) -> FsStateStore {
        FsStateStore::new(Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap())
    }

    #[test]
    fn second_mutation_lock_on_same_identity_fails_fast() {
        let temp = tempfile::tempdir().unwrap();
        let store = store(&temp);
        let id = identity("/w/a", "alpha");

        // Hold the advisory lock via the returned guard, then attempt a second
        // acquire on the SAME identity: it must not block, it must surface
        // OperationInProgress.
        let _guard = store.acquire(&id).unwrap();
        let err = store.acquire(&id).unwrap_err();
        assert!(
            matches!(err, StateStoreError::OperationInProgress { .. }),
            "expected OperationInProgress, got {err:?}"
        );
    }

    #[test]
    fn distinct_identities_acquire_concurrently() {
        let temp = tempfile::tempdir().unwrap();
        let store = store(&temp);
        let a = identity("/w/a", "alpha");
        let b = identity("/w/b", "beta");

        let _ga = store.acquire(&a).unwrap();
        // A different identity uses a different lock file and must not block.
        let _gb = store.acquire(&b).unwrap();
    }

    #[test]
    fn write_is_atomic_with_no_leftover_tmp() {
        let temp = tempfile::tempdir().unwrap();
        let store = store(&temp);
        let id = identity("/w/a", "alpha");
        store
            .write(&WorkspaceState {
                identity: id.clone(),
                state: SandboxState::Composed,
                failure: None,
                image_freshness: None,
                reconcile_fingerprint: None,
            })
            .unwrap();

        let state_path = store.path(&id);
        assert!(state_path.exists(), "state.json must be present");
        let dir = state_path.parent().unwrap();
        let leftovers: Vec<_> = fs::read_dir(dir.as_std_path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "no leftover .tmp: {leftovers:?}");
    }

    #[test]
    fn stale_lock_file_from_dead_owner_is_taken_over() {
        // A lock file left behind by a crashed owner carries no live `flock`
        // (the kernel released it on process death), so a fresh acquire must
        // succeed on that same file and overwrite the recorded owner PID —
        // without any racy unlink-based reclamation.
        let temp = tempfile::tempdir().unwrap();
        let store = store(&temp);
        let id = identity("/w/a", "alpha");
        let lock = store.lock_path(&id);
        fs::create_dir_all(lock.parent().unwrap().as_std_path()).unwrap();
        // A bogus, definitely-dead owner PID with no live lock behind it.
        fs::write(&lock, "4294967295\n").unwrap();

        let guard = store.acquire(&id).unwrap();
        assert!(lock.exists(), "lock file is reused, not unlinked");
        let recorded = fs::read_to_string(&lock).unwrap();
        assert_eq!(
            recorded.trim(),
            std::process::id().to_string(),
            "owner PID is overwritten with the live acquirer",
        );
        drop(guard);
    }

    #[test]
    fn mark_failed_writes_failed_state_without_forged_freshness() {
        let temp = tempfile::tempdir().unwrap();
        let store = store(&temp);
        let id = identity("/w/a", "alpha");
        store.mark_failed(&id, "boom".to_string()).unwrap();

        let read = store.read(&id).unwrap().expect("state present");
        assert_eq!(read.state, SandboxState::Failed);
        assert_eq!(read.failure.as_deref(), Some("boom"));
        assert!(read.image_freshness.is_none(), "no forged freshness");
    }

    #[test]
    fn distinct_identities_never_collide_on_separator_position() {
        // Separator-position differences must not alias to one key.
        assert_ne!(
            safe_key(&identity("/a-b", "c")),
            safe_key(&identity("/a/b", "c")),
        );
        assert_ne!(
            safe_key(&identity("a", "b_c")),
            safe_key(&identity("a_b", "c")),
        );
    }

    #[test]
    fn memory_store_round_trips() {
        let store = MemoryStateStore::default();
        let id = identity("/w/a", "alpha");
        assert!(store.read(&id).unwrap().is_none());
        store.mark_failed(&id, "nope".to_string()).unwrap();
        let read = store.read(&id).unwrap().expect("state present");
        assert_eq!(read.state, SandboxState::Failed);
        assert!(read.image_freshness.is_none());
    }
}
