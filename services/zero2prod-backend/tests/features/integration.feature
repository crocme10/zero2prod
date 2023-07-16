Feature: Integration

  @serial
  Scenario: Health Check
    When the user requests a health check
    Then the response is 200 OK

  @serial, @success
  Scenario: Successful subscription
    When the user subscribes with username "<username>" and email "<email>"
    Then the response is 200 OK
     And the database stored the username "<username>" and the email "<email>" with status "pending_confirmation"
     And the user receives an email with a confirmation link

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

  @serial, @success
  Scenario: Successful confirmation
    When the user subscribes with username "bob" and email "bob@acme.com"
     And the user retrieves the confirmation link
     And the user confirms his subscription with the confirmation link
    Then the response is 200 OK
     And the database stored the username "bob" and the email "bob@acme.com" with status "confirmed"

  @serial, @success
  Scenario: Double subscription
    When the user subscribes with username "bob" and email "bob@acme.com"
     And the user subscribes with username "bob" and email "bob@acme.com"
    Then the response is 200 OK
     And the user receives two confirmation emails

  @serial, @success
  Scenario: Unconfirmed subscribers don't receive the newsletter.
    When the user subscribes with username "bob" and email "bob@acme.com"
     And the admin notifies subscribers of a new issue of the newsletter
    Then no newsletter are sent
     And the response is 200 OK
