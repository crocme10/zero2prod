#!/usr/bin/env bash

set -Eeo pipefail

echo "------"
echo "${ZERO2PROD__DATABASE__HOST}"
echo "${ZERO2PROD__DATABASE__PORT}"
echo "${ZERO2PROD__DATABASE__USERNAME}"
echo "${ZERO2PROD__DATABASE__PASSWORD}"
echo "------"

_main() {
	if [ "$1" = 'run' ]; then
    /srv/zero2prod/bin/zero2prod --config-dir /srv/zero2prod/etc/zero2prod run
  fi
}

_main "$@"

