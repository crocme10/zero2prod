# Zero 2 Prod

'Zero To Production in Rust' is a book by Luca Palmieri where he details his
progress implementing a web application using Rust. It's a wonderful book, with
lots of resources, tips and tricks. It has fostered lots of projects, and so its
useful to browse through github and look at how developers have taken the book's
initial code and choices and made changes.

This is such a project, in which I have
- relied on the work of Luca,
- used bits and pieces in similar projects,
- used my own experience and made a couple of departures


+------------------------------------+--------------------------------------------+
| DIFFERENCE                         | MOTIVATION                                 |
+====================================+============================================+
| s / actix-web / axum / g           | [Motivation](/documentation/webserver.md)  |
+------------------------------------+--------------------------------------------+
| cucumber / gherkin for integration | [Motivation](/documentation/cucumber.md)   |
+------------------------------------+--------------------------------------------+

- I don't use any error library
- Using different database executor depending on the environment: Connection in
  Production, or Transaction in Testing.
- Using xtask (lifted from [Damccull](https://github.com/damccull/zero2prod.git))
- Maybe opentelemetry?
- Maybe frontend?
- Use speculoos instead of claim (unmaintained)
- Use mockall instead of wiremock

[![CI/CD Prechecks](https://github.com/crocme10/zero2prod/actions/workflows/general.yml/badge.svg)](https://github.com/crocme10/zero2prod/actions/workflows/general.yml)
[![Security audit](https://github.com/crocme10/zero2prod/actions/workflows/audit.yml/badge.svg)](https://github.com/crocme10/zero2prod/actions/workflows/audit.yml)

One to two paragraph statement about your product and what it does.

![](header.png)

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

### Compile from Source

The source for zero2prod uses sqlx, which makes compiles time checks against a
running database. So we have the choice,

- either use sqlx's offline mode, to use a file in the source, `sqlx-data.json`
  in place of a running database,
- or start the database.

Let's go with the second option. We use
[cargo xtask](https://github.com/matklad/cargo-xtask) to add automation tasks to
this project. The following command will start a docker container for postgres,
and run the migrations to setup the database.

```sh
cargo xtask postgres
```

This will take some time to build a part of the project (the common workspace)
which does not depend on the database, and the xtask workspace. It will run this
task. The database is configured with the files found in `./config/database`.

Now we should have a running database, we can compile the rest of the project.
sqlx depends on the environment variable `DATABASE_URL`, whose value can be
found at the last line of the previous command:

for bash users:

```sh
export DATABASE_URL="postgres://postgres:password@localhost:5462/newsletter"
```

or for fish users:

```sh
set DATABASE_URL="postgres://postgres:password@localhost:5462/newsletter"
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

Environment variables use '**' to separate sections, and ZERO2PROD for the
prefix. So to modify the database's name, which is the database.database_name
key, we should set ZERO2PROD**DATABASE\_\_DATABASE_NAME=newsletter

The server takes two command:

- config to display the configuration as JSON
- run to run the server.

In one terminal window, you monitor all tailwind:

```sh
npx tailwindcss -i tailwind.css -o css/main.css
```

```sh
curl --header "Content-Type: application/json" --request POST --data '{"username": "alice", "email": "alice@acme.inc"}' http://localhost:8082/subscriptions
```

In another terminal, you run

```sh
cargo leptos watch
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

## Release History

- 0.0.1
  - Work in progress

## Meta

Distributed under the MIT license. See `LICENSE` for more information.

[https://github.com/crocme10/zero2prod](https://github.com/crocme10/)

## Contributing

1. Fork it (<https://github.com/crocme10/zero2prod/fork>)
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request

<!-- Markdown link & img dfn's -->
