#!/usr/bin/env fish

# https://www.baeldung.com/openssl-self-signed-cert
mkdir -p certs

echo "Creating a private key and certificate signing request"

openssl req -x509 -newkey rsa:4096 -keyout certs/key.pem -out certs/cert.pem -sha256 -days 3650 -nodes -subj "/C=FR/ST=Paris/L=Paris/O=Area403/OU=IT Department/CN=example.com"
