FROM postgres:15.2

# Add user commands
ADD ./sql/1* /docker-entrypoint-initdb.d/

# Don't add dev commands
# ADD ./sql/2* /docker-entrypoint-initdb.d/
