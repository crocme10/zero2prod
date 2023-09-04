SELECT pg_terminate_backend(pid)
  FROM pg_stat_activity
  WHERE usename = 'bob' OR datname = 'newsletter';
DROP DATABASE IF EXISTS newsletter;
DROP USER IF EXISTS bob;

CREATE USER bob PASSWORD 'secret';
CREATE DATABASE newsletter owner bob ENCODING = 'UTF-8';
