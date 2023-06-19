# Zero 2 Prod

'Zero To Production in Rust' is a book by Luca Palmieri where he details his progress
implementing a web application using Rust. It's a wonderful book, with lots of resources,
tips and tricks. It has fostered lots of projects, and so its useful to browse through
github and look at how developers have taken the book's initial code and choices and made
changes.

This is such a project, where the main changes are:
* Using axum instead of actix-web (many developers seem to do that.)
* I use cucumber for integration testing.
* I don't use any error library
* Maybe xtask?
* Maybe opentelemetry?
* Maybe frontend?


[![CI/CD Prechecks](https://github.com/crocme10/zero2prod/actions/workflows/general.yml/badge.svg)](https://github.com/crocme10/zero2prod/actions/workflows/general.yml)
[![Security audit](https://github.com/crocme10/zero2prod/actions/workflows/audit.yml/badge.svg)](https://github.com/crocme10/zero2prod/actions/workflows/audit.yml)

One to two paragraph statement about your product and what it does.

![](header.png)

## Installation

OS X & Linux:

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

## Usage example

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
