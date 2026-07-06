use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) struct AllowEntry(pub(crate) String);

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Allowlist(pub(crate) Vec<AllowEntry>);

impl Allowlist {
    pub(crate) fn union(&self, other: &Self) -> Self {
        let mut out = self.0.clone();
        for entry in &other.0 {
            if !out.contains(entry) {
                out.push(entry.clone());
            }
        }
        Self(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_is_ordered_and_deduped() {
        let a = Allowlist(vec![AllowEntry("one".into()), AllowEntry("two".into())]);
        let b = Allowlist(vec![AllowEntry("two".into()), AllowEntry("three".into())]);
        assert_eq!(
            a.union(&b).0,
            vec![
                AllowEntry("one".into()),
                AllowEntry("two".into()),
                AllowEntry("three".into())
            ]
        );
    }
}
