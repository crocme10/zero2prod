# Architecture for Testing

When a system uses external services, we have often the following scenario:

Our code domain makes a call to a service. All the service knows about is its
interface, but it does not care about the implementation. This decoupling adds a
bit of friction (instead of the user calling directly the external service, we
have to wire an interface and an implementation) but we can then benefit from
being able to change the external service without having to change the domain.

![system](/documentation/img/system.png)

We have two testing scenarios:

## Testing Service Implementation

Here we want to make sure the implementation makes correct calls to the external
service.

We replace the external service with a wiremock::Server, and we inspect the
headers, path, arguments of the call to the wiremock::Server.

## Testing User code.

Here we want to make sure that the user code correctly calls the interface. So
we replace the service implementation with a Mockall implementation, which inspects
the function calls made to it. The user code is, in our case, the route handler.
