use serde::Serialize;

use crate::{
    domain::network::{AllowEntry, AllowEntryKind, Allowlist},
    error::AppError,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct NetworkPolicyArtifact {
    pub(crate) schema_version: u32,
    pub(crate) default_action: &'static str,
    pub(crate) rules: Vec<NetworkPolicyRule>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct NetworkPolicyRule {
    pub(crate) id: String,
    pub(crate) action: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) target: String,
    pub(crate) port: Option<u16>,
}

impl NetworkPolicyArtifact {
    pub(crate) fn from_allowlist(allowlist: &Allowlist) -> Result<Self, AppError> {
        let allowlist = allowlist.validate().map_err(AppError::validation)?;
        let mut rules = Vec::new();
        for (idx, entry) in allowlist.0.iter().enumerate() {
            let parsed = entry.parsed().map_err(AppError::validation)?;
            rules.push(NetworkPolicyRule {
                id: format!("allow-{:04}", idx + 1),
                action: "allow",
                kind: match parsed.kind {
                    AllowEntryKind::Host => "host",
                    AllowEntryKind::Cidr => "cidr",
                },
                target: parsed.target,
                port: parsed.port,
            });
        }
        Ok(Self {
            schema_version: crate::util::schema_version(),
            default_action: "drop",
            rules,
        })
    }

    pub(crate) fn serialize(&self) -> Result<String, AppError> {
        serde_json::to_string_pretty(self).map_err(|err| {
            AppError::unexpected(format!("failed to serialize network policy: {err}"))
        })
    }
}

pub(crate) fn artifact_from_strings(entries: &[String]) -> Result<NetworkPolicyArtifact, AppError> {
    let allowlist = Allowlist(
        entries
            .iter()
            .map(|entry| AllowEntry(entry.clone()))
            .collect(),
    );
    NetworkPolicyArtifact::from_allowlist(&allowlist)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_allowlist_is_default_drop_only() {
        let artifact = NetworkPolicyArtifact::from_allowlist(&Allowlist::default()).unwrap();
        assert_eq!(artifact.default_action, "drop");
        assert!(artifact.rules.is_empty());
    }

    #[test]
    fn each_canonical_entry_becomes_one_stable_allow_rule() {
        let artifact = NetworkPolicyArtifact::from_allowlist(&Allowlist(vec![
            AllowEntry("Example.COM:443".to_string()),
            AllowEntry("example.com:443".to_string()),
            AllowEntry("10.0.0.0/8".to_string()),
        ]))
        .unwrap();
        assert_eq!(artifact.rules.len(), 2);
        assert_eq!(artifact.rules[0].id, "allow-0001");
        assert_eq!(artifact.rules[0].target, "example.com");
        assert_eq!(artifact.rules[0].port, Some(443));
        assert_eq!(artifact.rules[1].kind, "cidr");
    }

    #[test]
    fn serialization_is_deterministic() {
        let artifact = artifact_from_strings(&["github.com:443".to_string()]).unwrap();
        insta::assert_snapshot!("network_policy__github_443", artifact.serialize().unwrap());
    }
}
