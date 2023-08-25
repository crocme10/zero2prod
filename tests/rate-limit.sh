#!/usr/bin/env bash

for i in {1..40}
do
  curl -v --header "Content-Type: application/json" \
    --data '{"username":"xyz","password":"xyz"}' \
    http://localhost:8084/api/v1/login
done
