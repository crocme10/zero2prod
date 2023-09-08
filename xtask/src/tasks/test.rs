use std::process::{Command, ExitStatus};

use crate::{check_nextest_exists, project_root};

pub fn xtest() -> Result<(), anyhow::Error> {
    println!("Running unit tests...");
    run_unit_test()?;
    println!("Running integration tests...");
    run_integration_test()?;
    Ok(())
}

pub fn run_unit_test() -> Result<ExitStatus, anyhow::Error> {
    // let test = if check_nextest_exists().is_ok() {
    //     Command::new("cargo")
    //         .current_dir(project_root())
    //         .args(["nextest", "run", "--lib", "--bins"])
    //         .status()?
    // } else {
    let test = Command::new("cargo")
        .current_dir(project_root())
        .args(["test", "-p", "zero2prod-backend", "--lib", "--bins"])
        .status()?;
    //};
    Ok(test)
}

pub fn run_integration_test() -> Result<ExitStatus, anyhow::Error> {
    let test = Command::new("cargo")
        .current_dir(project_root())
        .args(["test", "--test", "integration"])
        .status()?;
    Ok(test)
}
