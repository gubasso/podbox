pub(crate) mod seed;

pub(crate) fn schema_version() -> u32 {
    1
}

pub(crate) fn anyhow_ok() -> anyhow::Result<()> {
    Ok(())
}
