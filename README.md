# Zero 2 Prod

## Background

[Zero To Production in Rust](https://www.zero2prod.com) is a book by Luca
Palmieri where he details his progress implementing a web backend using Rust.
It's a wonderful book, with lots of resources, tips and tricks. It has fostered
lots of projects, and so its useful to browse through github and look at how
developers have taken the book's initial code and choices and made changes.

This is such a project, in which I have

- relied heavily on the work of Luca Palmieri for the structure and process,
- incorporated bits and pieces in similar projects,
- used my own experience to make a couple of departures

| DIFFERENCE FROM ORIGINAL           | MOTIVATION                                               |
| ---------------------------------- | -------------------------------------------------------- |
| s / actix-web / axum / g           | [Motivation](/documentation/webserver.md)                |
| cucumber / gherkin for integration | [Motivation](/documentation/cucumber.md)                 |
| anyhow > /dev/null                 | [Motivation](/documentation/error-handling.md)           |
| 'generic' database executor        | [Motivation](/documentation/database.md)                 |
| Using xtask                        | [Description](/documentation/xtasks.md)                  |
| Traits for improved testing        | [Motivation](/documentation/architecture-for-testing.md) |
| s / claim / speculoos / g          | claim does not seem to be maintained                     |
| Maybe opentelemetry?               |                                                          |
| frontend                           | vue.js                                                   |

[![CI/CD Prechecks](https://github.com/crocme10/zero2prod/actions/workflows/general.yml/badge.svg)](https://github.com/crocme10/zero2prod/actions/workflows/general.yml)
[![Security audit](https://github.com/crocme10/zero2prod/actions/workflows/audit.yml/badge.svg)](https://github.com/crocme10/zero2prod/actions/workflows/audit.yml)

## Introduction

This project implements the backend and the frontend of a newsletter website

## Installation

### Requirements

This is a rust application, so we need to have the following setup:

- The [rust toolchain](https://www.rust-lang.org/tools/install)
- git
- docker
- openssl (generate certificates)
- npm (build the frontend)

To install this website, we need to setup 3 components:

1. A database, where we store all the details.
2. The backend, which is webserver providing a REST API.
3. The frontend, which is a client-side rendering webapplication

### Database

Clone the repository:

```sh
git clone https://github.com/crocme10/zero2prod
```

The source for zero2prod uses sqlx, which makes compiles time checks against a
running database. So we have the choice,

- either use sqlx's offline mode (requires to run `cargo sqlx prepare` ahead of
  the compilation)
- or start the database before compiling the project.

Let's go with the second option. We use
[cargo xtask](https://github.com/matklad/cargo-xtask) to add automation tasks to
this project. The TLDR is `cargo install cargo-xtask`. Then the following
command will start a docker container for postgres, and run the migrations to
setup the database.

```sh
cargo xtask postgres
```

This will take some time to build a part of the project (the common workspace)
which does not depend on the database (otherwise, it would fail because the
database is not there yet), and the xtask workspace. The database is configured
with the files found in `./config/database`. It will launch a docker container
with postgres, and run the migrations to setup the database.

Now we should have a running database, we can compile the rest of the project.
sqlx knows about the database through the environment variable `DATABASE_URL`,
whose value can be found at the last line of the previous command:

for bash users:

```sh
export DATABASE_URL="postgres://postgres:password@localhost:5462/newsletter"
```

or for fish users:

```fish
set DATABASE_URL "postgres://postgres:password@localhost:5462/newsletter"
```

### Frontend

The frontend will compile into a directory, that will be served by the frontend.

```sh
cargo xtask frontend
```

### Backend

The database is a prerequisite for compiling the backend, because sqlx makes
compile time checks against the database. So with the `DATABASE_URL` set, we
can compile the application:

```sh
cargo build --release
```

You may also need to generate certificates:

```sh
cargo xtask certificate
```

### Build for Docker

At the root of the project:

```sh
docker build -t zero2prod:latest .
```

## Deployment

[Details](/documentations/deployment.md)

## Usage

### Server

When you build the server (debug or release), you get a binary in target/{debug,
release}/zero2prod

This binay can be configured with configuration files found in ./config

The binary requires a path to a configuration directory where all configuration
is found.

This configuration can be overriden with local configuration files, and with
environment variables.

Environment variables use `__` to separate sections, and ZERO2PROD for the
prefix. So to modify the database's name, which is in the database section, we
should set `ZERO2PROD\_\_DATABASE\_\_DATABASE_NAME=newsletter`

The server takes two commands:

- **config**: to display the configuration as JSON
- **run**: to run the server.

```sh
./target/debug/zero2prod -c ./config run
```

If you want to modify configuration, you can easily do that on the command line:

```sh
./target/debug/zero2prod -c ./config -s database.require_ssl=false -s application.http=8080 -s application.https=8443 run
```

```sh
curl --header "Content-Type: application/json" --request POST --data '{"username": "alice", "email": "alice@acme.inc"}' http://localhost:8082/subscriptions
```

## Development setup

Start by deploying a postgres docker container:

```sh
cargo xtask postgres
```

Then set your DATABASE_URL variable accordingly

And then build run the tests.

```sh
cargo xtask ci

```

## File Organization

zero2prod/ ├─ services/ │ ├─ zero2prod_backend/ backend server │ ├─
zero2prod_frontend/ frontend wasm │ ├─ zero2prod_common/ structures shared by
backend and frontend │ ├─ zero2prod_fakeemail/ simple server to mock external
email service. ├─ config/ configuration ├─ docker/ Dockerfiles and entrypoints
├─ documentation/ Additional documentation ├─ common/ code shared between xtask
and services ├─ xtask/ tasks implementation. ├─ dev.sh/ script to start services
├─ spec.yaml/ digital ocean deployment

## Release History

- 0.0.4

- 0.0.1
  - Work in progress

## License

Distributed under the MIT license. See `LICENSE` for more information.

[https://github.com/crocme10/zero2prod](https://github.com/crocme10/)

## Contributing

1. Fork it (<https://github.com/crocme10/zero2prod/fork>)
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request

<!-- Markdown link & img dfn's -->
