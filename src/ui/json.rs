use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct Versioned<T: Serialize> {
    pub(crate) schema_version: u32,
    #[serde(flatten)]
    pub(crate) value: T,
}

pub(crate) fn versioned<T: Serialize>(value: T) -> Versioned<T> {
    Versioned {
        schema_version: 1,
        value,
    }
}
