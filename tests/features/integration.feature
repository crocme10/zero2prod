Feature: Integration

  Background:
    Given the service has been started

  Scenario: When the user calls the health_check endpoint, we get a 200 Ok response
    When the user requests a health check
    Then the response is 200 OK

  Scenario: When the user calls the subscriptions endpoint, we get a 200 Ok response
    When the user subscribes with username "<username>" and email "<email>"
    Then the response is 200 OK

    Examples:
      | username         | email                     |
      | bob              | bob@acme.com              |

  Scenario: When the user calls the subscriptions endpoint with incomplete credentials, we get a 400 Bad Request response
    When the user subscribes with username "<username>" and email "<email>"
    Then the response is 400 Bad Request

    Examples:
      | username         | email                     |
      |                  | bob@acme.com              |
      | bob              |                           |
      |                  |                           |

