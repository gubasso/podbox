use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) struct Digest(String);

impl Digest {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DigestInput {
    parts: Vec<(String, Vec<u8>)>,
}

impl DigestInput {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn part(mut self, label: impl Into<String>, bytes: impl AsRef<[u8]>) -> Self {
        self.parts.push((label.into(), bytes.as_ref().to_vec()));
        self
    }

    pub(crate) fn finish(&self) -> Digest {
        let mut hasher = Sha256::new();
        for (label, bytes) in &self.parts {
            hasher.update(label.as_bytes());
            hasher.update([0]);
            hasher.update(bytes.len().to_be_bytes());
            hasher.update(bytes);
            hasher.update([0xff]);
        }
        Digest(format!("sha256:{}", hex::encode(hasher.finalize())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn known_vector_is_stable() {
        let digest = DigestInput::new()
            .part("a", b"hello")
            .part("b", b"world")
            .finish();
        assert_eq!(
            digest.as_str(),
            "sha256:e192a3ac8d47a8dc52ca97c6193282d7916023069567f058f8858099c64fd79c"
        );
    }

    #[test]
    fn order_is_part_of_digest() {
        let a = DigestInput::new().part("a", b"1").part("b", b"2").finish();
        let b = DigestInput::new().part("b", b"2").part("a", b"1").finish();
        assert_ne!(a, b);
    }

    proptest! {
        #[test]
        fn identical_inputs_match(
            label in "[a-z]{1,8}",
            bytes in proptest::collection::vec(any::<u8>(), 0..32),
        ) {
            let a = DigestInput::new().part(&label, &bytes).finish();
            let b = DigestInput::new().part(&label, &bytes).finish();
            prop_assert_eq!(a, b);
        }
    }
}
