use std::net::ToSocketAddrs;
use std::{fmt, net::TcpListener};

#[derive(Debug)]
pub enum Error {
    AddressResolution {
        context: String,
    },
    AddressDefinition {
        context: String,
        source: std::io::Error,
    },
    TcpListener {
        context: String,
        source: std::io::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AddressResolution { context } => {
                write!(
                    fmt,
                    "REST Server: Could not resolve server address: {context}",
                )
            }
            Error::AddressDefinition { context, source } => {
                write!(fmt, "Could not build client request: {context} | {source}")
            }
            Error::TcpListener { context, source } => {
                write!(fmt, "Could not build TCP listener: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

pub fn listen_with_host_port(host: &str, port: u16) -> Result<TcpListener, Error> {
    let addr = (host, port);
    let addr = addr
        .to_socket_addrs()
        .map_err(|err| Error::AddressDefinition {
            context: format!("REST Server: Could not resolve address  {}:{}", host, port),
            source: err,
        })?
        .next()
        .ok_or_else(|| Error::AddressResolution {
            context: format!("REST Server: Could not resolve address  {}:{}", host, port),
        })?;

    TcpListener::bind(addr).map_err(|err| Error::TcpListener {
        context: format!("REST Server: Could not listen on address {}:{}", host, port),
        source: err,
    })
}
