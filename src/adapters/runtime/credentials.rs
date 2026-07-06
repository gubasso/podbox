use camino::Utf8PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CredentialPolicy {
    pub(crate) tmpfs_destination: Utf8PathBuf,
    deny_sources: Vec<Utf8PathBuf>,
}

impl CredentialPolicy {
    pub(crate) fn ephemeral() -> Self {
        Self {
            tmpfs_destination: Utf8PathBuf::from("/run/podbox/credentials"),
            deny_sources: vec![
                Utf8PathBuf::from("~/.ssh"),
                Utf8PathBuf::from("~/.aws"),
                Utf8PathBuf::from("~/.config/gcloud"),
                Utf8PathBuf::from("~/.docker"),
                Utf8PathBuf::from("/run/user"),
            ],
        }
    }

    pub(crate) fn mount_args(&self) -> Vec<String> {
        vec![format!(
            "--mount=type=tmpfs,destination={}",
            self.tmpfs_destination
        )]
    }

    pub(crate) fn deny_sources(&self) -> &[Utf8PathBuf] {
        &self.deny_sources
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_policy_uses_tmpfs_and_excludes_deny_list_sources() {
        let policy = CredentialPolicy::ephemeral();
        let rendered = policy.mount_args().join(" ");
        assert!(rendered.contains("type=tmpfs"));
        assert!(rendered.contains("/run/podbox/credentials"));
        for denied in policy.deny_sources() {
            assert!(!rendered.contains(denied.as_str()));
        }
    }
}
