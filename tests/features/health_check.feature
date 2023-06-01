Feature: Health Checks

  Scenario: When the user calls the health_check endpoint, we get a 200 Ok response
    Given the service has been started
    When the user requests a health check
    Then the response is 200 OK
