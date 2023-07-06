# Deployment

## Digital Ocean Kubernetes

Because we're deploying multiple component to integrate both frontend and backend,
I decided to change the original DO App Platform deployment, to a more complex one.

Each service is a Docker container:

* **core**: The backend rust application (a REST API)
* **database**: The postgresql database
* **web**: The static site
* **frontend**: An Nginx router.


## (historical) Digital Ocean App Platform

The app is deployed on Digital Ocean (DO)'s' "App platform", in a continuous
delivery setup. This means every time a push (TODO Validated / Tested ?) occurs
on the main branch, DO pulls the new version, builds it, and redeploys it.

The specification for the app is in `/spec.yaml`

The app is deployed as a docker container, so we find
`dockerfile_path: Dockerfile`

### Preflight checks

#### Docker Build

You must successfully be able to build and run your app as a docker container:

```sh
docker build -t zero2prod:latest .
```

#### Docker Stack

You must be able to use this docker image just built together with a dockerized
database.

We use docker stack to run some tests...

The stack.yaml uses environment variables to configure the database parameters
for the webapp, because this is what will be used for the Digital Ocean's App
Plaftorm.

Start the docker stack

```sh
docker stack deploy -c docker/stack.yml zero2prod
```

How is the stack running?

After a couple seconds, I run

```sh
docker stack ps zero2prod
k01xjpdpqc3s   zero2prod_db.1              postgres:latest    taipan    Running         Preparing 12 seconds ago
vtty05k72bc4   zero2prod_zero2prod.1       zero2prod:latest   taipan    Ready           Ready 1 second ago
ybaw4emgoxen    \_ zero2prod_zero2prod.1   zero2prod:latest   taipan    Shutdown        Complete 3 seconds ago
iezn8qfmzoga    \_ zero2prod_zero2prod.1   zero2prod:latest   taipan    Shutdown        Complete 9 seconds ago
```

Something does not seem to work... zero2prod is starting and shuting down
immediately. What's going on?

Check the logs

```sh
docker service logs -f zero2prod_zero2prod
zero2prod_zero2prod.1.cnz0cuzxvl60@taipan    | Hello, world!
[...]
```

Something is _very_ wrong.... Nowhere in my code do I have the string 'Hello,
world!'

So, stop the stack:

```sh
docker stack rm zero2prod
```

and run the zero2prod container interactively to explore...

Since I might have to do that a few times, and I need environment variables to
parameterize the connection to the postgres database, I store the environment
variables in the `.env`, and run `load-dot-env .env` since I use fish.
`env | grep ZERO2PROD` tells me the environment variables have been correctly
set, so lets start the container...

```sh
docker run -it zero2prod:latest
Hello, world!
```

Ok, our worst fears are confirmed!

And then, after a few minutes of wonderment.... I understand what's going on....
The 'Hello, world!' is not my code, it's the default binary when creating a new
rust project. So the problem is with the Dockerfile...

After fixing the Dockerfile, the previous command looks more like what its
supposed to be, albeit not a full success:

```sh
docker run -it zero2prod:latest
Error: Storage { context: "Establishing a database connection", source: Connection { context: "PostgreSQL Storage: Could not establish a connection", source: Io(Os { code: 99, kind: AddrNotAvailable, message: "Cannot assign requested address" }) } }
```

Let's fix that connection issue... First let's see what parameters zero2prod
uses to connect to the database:

```sh
./target/debug/zero2prod -c ./config config
{
  "network": {
    "host": "0.0.0.0",
    "port": 8080
  },
  "database": {
    "username": "bob",
    "password": "secret",
    "port": 5435,
    "host": "db",
    "database_name": "newsletter",
    "require_ssl": true,
    "connection_timeout": 2000,
    "executor": "connection"
  },
  "mode": "default"
}
```

This confirms that environment variables are read correctly... Now lets try to
manually connect with those parameters (Note that the docker container with the
database should still be running).

## Digital Ocean 
Run some requests against the web app's API.
