use std::process::{Command, ExitStatus};

use owo_colors::OwoColorize;

use crate::{
    project_root,
    tasks::test::{run_integration_test, run_unit_test},
};

pub fn ci() -> Result<(), anyhow::Error> {
    println!("Running `cargo check`...");
    let check = Command::new("cargo")
        .current_dir(project_root())
        .args(["check", "--all"])
        .status()?;

    println!("Running `cargo clippy`...");
    let clippy = Command::new("cargo")
        .current_dir(project_root())
        .args(["clippy", "--all"])
        .status()?;

    println!("Running `cargo build`...");
    let build = Command::new("cargo")
        .current_dir(project_root())
        .args(["build", "--all"])
        .status()?;

    println!("Running unit tests...");
    let unit_test = run_unit_test()?;

    println!("Running integration tests...");
    let integration_test = run_integration_test()?;

    println!("Running `cargo deny`...");
    let audit = Command::new("cargo")
        .current_dir(project_root())
        .args(["deny", "check"])
        .status()?;

    println!("Running `cargo fmt`...");
    let fmt = Command::new("cargo")
        .current_dir(project_root())
        .args(["fmt"])
        .status()?;

    println!("Running `cargo sqlx prepare --workspace --check -- --lib`...");
    let sqlx_prep = Command::new("cargo")
        .current_dir(project_root())
        // .env("DATABASE_URL", settings.connection_string())
        .args(["sqlx", "prepare", "--check", "--workspace"])
        .status()?;

    print_error_with_status_code("cargo check", check);
    print_error_with_status_code("cargo clippy", clippy);
    print_error_with_status_code("cargo build", build);
    print_error_with_status_code("unit tests", unit_test);
    print_error_with_status_code("integration tests", integration_test);
    print_error_with_status_code("cargo deny", audit);
    print_error_with_status_code("cargo fmt", fmt);
    print_error_with_status_code("cargo sqlx prepare", sqlx_prep);

    println!(
        "CI checks complete. Consider running `cargo xtask coverage`.\
    Coverage checks are not completed by the CI checks due to the time requirement."
    );
    Ok(())
}

fn print_error_with_status_code(task: &str, status: ExitStatus) {
    let code = match status.code() {
        Some(x) => x.to_string(),
        None => "<< no status code >>".to_string(),
    };
    if !status.success() {
        println!(
            "{} `{}` finished with a non-zero status code: {}",
            "Error:".to_string().red(),
            task.blue(),
            code
        );
    }
}
