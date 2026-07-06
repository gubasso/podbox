use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
pub(crate) struct Provenance {
    pub(crate) keys: BTreeMap<String, String>,
}

impl Provenance {
    pub(crate) fn record(&mut self, key: impl Into<String>, source: impl Into<String>) {
        self.keys.insert(key.into(), source.into());
    }

    pub(crate) fn source(&self, key: &str) -> &str {
        self.keys.get(key).map(String::as_str).unwrap_or("default")
    }
}
