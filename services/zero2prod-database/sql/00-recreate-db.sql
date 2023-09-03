SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE username='bob' OR datname=''
 
CREATE USE bob PASSWORD 'secret';

CREATE DATABASE newsletter OWNER bob ENCODING = 'UTF-8';
