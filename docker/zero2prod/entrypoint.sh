#!/usr/bin/env bash

set -Eeo pipefail

echo "${newsletter.HOSTNAME}"
echo "${newsletter.PORT}"
echo "${newsletter.USERNAME}"
echo "${newsletter.PASSWORD}"
echo "------"
echo "${ZERO2PROD__DATABASE_HOST}"
echo "${ZERO2PROD__DATABASE_PORT}"
echo "${ZERO2PROD__DATABASE_USERNAME}"
echo "${ZERO2PROD__DATABASE_PASSWORD}"
echo "------"

_main() {
	if [ "$1" = 'run' ]; then
    /srv/zero2prod/bin/zero2prod --config-dir /srv/zero2prod/etc/zero2prod config
  fi
}

_main "$@"

