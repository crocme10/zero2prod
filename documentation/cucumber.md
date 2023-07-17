# Cucumber for Integration Testing

## Motivations

The test specifications act as a contract between multiple parties (eg project
manager, developer, tester, ...). This contract should be comprehensive and
written in a language understood by all. This is what cucumber provides through
its [Gherkin Syntax](https://cucumber.io/docs/gherkin/reference/)


## Integration Tests

When your development environment is available, just run

```sh
cargo test --test integration
[...]
    Finished test [unoptimized + debuginfo] target(s) in 9.82s
     Running tests/integration.rs (target/debug/deps/integration-3e61feee4bff2b04)
   ✔  When the user requests a health check
   ✔  When a valid subscriber with username "bob" and email "bob@acme.com" registers
   ✔  When an invalid subscriber with username "" and email "bob@acme.com" registers
[...]
```

If there is a problem with a test...

1. Identify in which scenario the error occurs (open the feature file)
2. Run that scenario using trace:
  You use `--trace true`, and `--name` followed by an expression that is unique for
  that scenario, for example:

  ```sh
  cargo test --test integration -- --trace true --name 'Confirmed subscribers'
  ```
