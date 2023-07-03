Feature: Integration

  @serial
  Scenario: When the user calls the health_check endpoint, we get a 200 Ok response
    When the user requests a health check
    Then the response is 200 OK

  @serial, @success
  Scenario: Successful subscription
    When the user subscribes with username "<username>" and email "<email>"
    Then the response is 200 OK
     And the database stored the username "<username>" and the email "<email>"
     And the user receives an email with a confirmation link.

    Examples:
      | username         | email                     |
      | bob              | bob@acme.com              |

  @serial, @failure
  Scenario: Subscription with invalid credentials
    When the user subscribes with username "<username>" and email "<email>"
    Then the response is 400 Bad Request

    Examples:
      | username         | email                     |
      |                  | bob@acme.com              |
      | bob              |                           |
      |                  |                           |
      | bob              | not-an-email              |

