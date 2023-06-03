# Github CI

## Tests

### Integration Tests

Integration tests require a database connection. For the Github CI, we perform the following tasks:

* start a postgres service. This is automatic, since we have a 'services' section in the workflow.
* initialize the database, using the sqlx cli migration tool.
* prior to running the integration tests, we establish a connection to the database. We use a 
  special configuration file (`config/database/github.toml`)
