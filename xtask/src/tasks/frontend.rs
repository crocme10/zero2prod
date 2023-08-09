use std::process::{Command, ExitStatus};

use crate::{check_npm_exists, project_root};

pub fn frontend() -> Result<(), anyhow::Error> {
    println!("Building frontend...");
    build_frontend()?;
    Ok(())
}

pub fn build_frontend() -> Result<ExitStatus, anyhow::Error> {
    let install = if check_npm_exists().is_ok() {
        Command::new("npm")
            .current_dir(project_root().join("services").join("zero2prod-frontend"))
            .args(["install"])
            .status()?
    } else {
        anyhow::bail!("Unable to run npm install. npm is not available.");
    };
    if install.success() {
        let build = if check_npm_exists().is_ok() {
            Command::new("npm")
                .current_dir(project_root().join("services").join("zero2prod-frontend"))
                .args(["run"])
                .args(["build"])
                .status()?
        } else {
            anyhow::bail!("Unable to run npm install. npm is not available.");
        };
        Ok(build)
    } else {
        Ok(install)
    }
}
