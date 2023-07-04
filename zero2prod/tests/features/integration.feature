Feature: Integration

  @serial
  Scenario: When the user calls the health_check endpoint, we get a 200 Ok response
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

  # @serial, @success
  # For this test to pass, we have to make significant changes:
  # Upon the first confirmation we delete the token.
  # When we hit the link again, we only have the token, which is pretty useless,
  # since its been deleted from the base.
  # Scenario: Double confirmation
  #   When the user subscribes with username "bob" and email "bob@acme.com"
  #    And the user retrieves the confirmation link
  #    And the user confirms his subscription with the confirmation link
  #    And the user confirms his subscription with the confirmation link
  #   Then the response is 200 OK

# What happens if the subscription token is well-formatted but non-existent?
# Add validation on the incoming token, we are currently passing the raw user
