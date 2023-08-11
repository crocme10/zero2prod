Feature: Health

  @serial
  Scenario: Health Check
    When the user requests a health check
    Then the response is 200 OK
