mod initialisation;
mod utilities;

use std::io::Write;
use anyhow::Result;
use validator::ValidateEmail;
use rand::Rng;
use std::error::Error;

fn main() -> Result<()> {
    utilities::clear_screen();
    std::io::stdout().flush()?;
    initialisation::check_initial("./");
    Ok(())
}











