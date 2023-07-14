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
| Maybe frontend?                    |                                                          |

[![CI/CD Prechecks](https://github.com/crocme10/zero2prod/actions/workflows/general.yml/badge.svg)](https://github.com/crocme10/zero2prod/actions/workflows/general.yml)
[![Security audit](https://github.com/crocme10/zero2prod/actions/workflows/audit.yml/badge.svg)](https://github.com/crocme10/zero2prod/actions/workflows/audit.yml)

## Introduction

This project implements the backend of a newsletter website

## Installation

### Requirements

This is a rust application, so we need to have the following setup:

- The [rust toolchain](https://www.rust-lang.org/tools/install)
- Git
- Docker

### From Source

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
this project. The following command will start a docker container for postgres,
and run the migrations to setup the database.

```sh
cargo xtask postgres
```

This will take some time to build a part of the project (the common workspace)
which does not depend on the database (otherwise, it would fail because the
database is not there yet), and the xtask workspace. The database is configured
with the files found in `./config/database`.

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

Then compile zero2prod:

```sh
cargo build
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
prefix. So to modify the database's name, which is in the database section,
we should set `ZERO2PROD\_\_DATABASE\_\_DATABASE_NAME=newsletter`

The server takes two commands:

- **config**: to display the configuration as JSON
- **run**: to run the server.

```sh
./target/debug/zero2prod -c ./config run
```

If you want to modify configuration, you can easily do that on the command line:

```sh
./target/debug/zero2prod -c ./config -s database.require_ssl=false -s application.port=8082 run
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


zero2prod/
├─ services/
│  ├─ zero2prod_backend/           backend server
│  ├─ zero2prod_frontend/          frontend wasm
│  ├─ zero2prod_common/            structures shared by backend and frontend
│  ├─ zero2prod_fakeemail/         simple server to mock external email service.
├─ config/                         configuration
├─ docker/                         Dockerfiles and entrypoints
├─ documentation/                  Additional documentation
├─ common/                         code shared between xtask and services
├─ xtask/                          tasks implementation.
├─ dev.sh/                         script to start services 
├─ spec.yaml/                      digital ocean deployment

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
