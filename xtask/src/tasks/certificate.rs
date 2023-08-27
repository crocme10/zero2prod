use std::process::{Command, ExitStatus};

use crate::{check_openssl_exists, project_root};

pub fn certificate() -> Result<(), anyhow::Error> {
    println!("Generating certificate...");
    generate_certificate()?;
    Ok(())
}

pub fn generate_certificate() -> Result<ExitStatus, anyhow::Error> {
    let install = if check_openssl_exists().is_ok() {
        let path = project_root()
            .join("services")
            .join("zero2prod-backend")
            .join("certs");
        let e = path.try_exists()?;
        if !e {
            std::fs::create_dir_all(&path)?;
        }
        Command::new("openssl")
            .current_dir(project_root().join("services").join("zero2prod-backend"))
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:4096",
                "-sha256",
                "-days",
                "365",
                "-nodes",
                "-keyout",
                "certs/key.pem",
                "-out",
                "certs/cert.pem",
                "-subj",
                "/CN=zero2prod",
                "-addext",
                "subjectAltName=DNS:example.com,DNS:*.example.com,IP:10.0.0.1",
            ])
            .status()?
    } else {
        anyhow::bail!("Unable to run openssl. openssl is not available.");
    };
    Ok(install)
}
