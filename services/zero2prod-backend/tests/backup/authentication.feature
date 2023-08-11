Feature: Authentication

  @serial, @success
  Scenario: Successful registration
    When a user registers
    Then the response is 200 OK
     And a user can successfully login
