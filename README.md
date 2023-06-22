# Zero 2 Prod

'Zero To Production in Rust' is a book by Luca Palmieri where he details his
progress implementing a web application using Rust. It's a wonderful book, with
lots of resources, tips and tricks. It has fostered lots of projects, and so its
useful to browse through github and look at how developers have taken the book's
initial code and choices and made changes.

This is such a project, where the main changes are:

- Using axum instead of actix-web (many developers seem to do that.)
- I use cucumber for integration testing.
- I don't use any error library
- Maybe xtask?
- Maybe opentelemetry?
- Maybe frontend?

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

This will take some time to build a part of the project (the common workspace) which
does not depend on the database, and the xtask workspace. It will run this task. The
database is configured with the files found in `./config/database`.

Now we should have a running database, we can compile the rest of the project. sqlx
depends on the environment variable `DATABASE_URL`, whose value can be found at the
last line of the previous command:

for bash users:

```sh
export DATABASE_URL="postgres://postgres:password@localhost:5462/newsletter"
```

or for fish users:

```sh
set DATABASE_URL="postgres://postgres:password@localhost:5462/newsletter"
```

```sh
cargo install --locked cargo-leptos
# Not sure cargo install trunk is useful anymore.
npm install -D tailwindcss
```

By default, `cargo-leptos` uses `nightly` Rust, `cargo-generate`, and `sass`. If
you run into any trouble, you may need to install one or more of these tools.

1. `rustup toolchain install nightly --allow-downgrade` - make sure you have
   Rust nightly
2. `rustup default nightly` - setup nightly as default, or you can use
   rust-toolchain file later on
3. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust
   to WebAssembly
4. `cargo install cargo-generate` - install `cargo-generate` binary (should be
   installed automatically in future)
5. `npm install -g sass` - install `dart-sass` (should be optional in future

Windows:

?

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

Describe how to install all development dependencies and how to run an automated
test-suite of some kind. Potentially do this for multiple platforms.

```sh
make install
npm test
```

## Release History

- 0.0.1
  - Work in progress

## Meta

YourEmail@example.com

Distributed under the XYZ license. See `LICENSE` for more information.

[https://github.com/yourname/github-link](https://github.com/dbader/)

## Contributing

1. Fork it (<https://github.com/yourname/yourproject/fork>)
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request

<!-- Markdown link & img dfn's -->

[npm-image]: https://img.shields.io/npm/v/datadog-metrics.svg?style=flat-square
[npm-url]: https://npmjs.org/package/datadog-metrics
[npm-downloads]:
  https://img.shields.io/npm/dm/datadog-metrics.svg?style=flat-square
[travis-image]:
  https://img.shields.io/travis/dbader/node-datadog-metrics/master.svg?style=flat-square
[travis-url]: https://travis-ci.org/dbader/node-datadog-metrics
[wiki]: https://github.com/yourname/yourproject/wiki
