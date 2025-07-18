use anyhow::Result;
use vergen_gitcl::{Emitter, GitclBuilder};

pub fn main() -> Result<()> {
    Emitter::default()
        .add_instructions(&GitclBuilder::all_git()?)?
        .emit()?;
    Ok(())
}
