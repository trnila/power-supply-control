use anyhow::Result;
use vergen::EmitBuilder;

pub fn main() -> Result<()> {
    EmitBuilder::builder()
        .git_describe(true, true, None)
        .emit()?;
    Ok(())
}
