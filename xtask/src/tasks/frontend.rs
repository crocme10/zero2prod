use std::process::{Command, ExitStatus};

use crate::{check_trunk_exists, project_root};

pub fn frontend() -> Result<(), anyhow::Error> {
    println!("Building frontend...");
    build_frontend()?;
    Ok(())
}

pub fn build_frontend() -> Result<ExitStatus, anyhow::Error> {
    let test = if check_trunk_exists().is_ok() {
        Command::new("trunk")
            .current_dir(project_root().join("services").join("zero2prod-frontend"))
            .args(["build"])
            .status()?
    } else {
        anyhow::bail!("Unable to run trunk build. trunk is not available.");
    };
    Ok(test)
}
