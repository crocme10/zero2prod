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
        .args(["check", "-p", "zero2prod"])
        .status()?;

    println!("Running `cargo clippy`...");
    let clippy = Command::new("cargo")
        .current_dir(project_root())
        .args(["clippy", "-p", "zero2prod"])
        .status()?;

    println!("Running `cargo build`...");
    let build = Command::new("cargo")
        .current_dir(project_root())
        .args(["build", "-p", "zero2prod"])
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

    println!("Running `cargo sqlx prepare --check -- --lib`...");
    // The sqlx-data.json file is expected at the root of the project.
    // But, for some reason, it seems to be generated from the zero2prod directory.
    // So we move it to that directory, update it, and then move it back to the
    // project's root.
    // TODO This might change with version v0.7.0, with the --workspace argument.
    // FIXME This is covered in the xtask prepare.
    let mv_sqlx_data = Command::new("mv")
        .current_dir(project_root())
        .args(["sqlx-data.json", "zero2prod/"])
        .status()?;
    let sqlx_prep = Command::new("cargo")
        .current_dir(project_root().join("zero2prod"))
        .args(["sqlx", "prepare", "--check", "--", "--lib"])
        .status()?;
    let mv_sqlx_data_back = Command::new("mv")
        .current_dir(project_root())
        .args(["zero2prod/sqlx-data.json", "."])
        .status()?;
    print_error_with_status_code("cargo check", check);
    print_error_with_status_code("cargo clippy", clippy);
    print_error_with_status_code("cargo build", build);
    print_error_with_status_code("unit tests", unit_test);
    print_error_with_status_code("integration tests", integration_test);
    print_error_with_status_code("cargo deny", audit);
    print_error_with_status_code("cargo fmt", fmt);
    print_error_with_status_code("mv sqlx-data.json zero2prod/", mv_sqlx_data);
    print_error_with_status_code("cargo sqlx prepare", sqlx_prep);
    print_error_with_status_code("mv zero2prod/sqlx-data.json .", mv_sqlx_data_back);

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
