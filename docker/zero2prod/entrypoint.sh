#!/usr/bin/env bash

set -Eeo pipefail

_main() {
	if [ "$1" = 'run' ]; then
    /srv/zero2prod/bin/zero2prod --config-dir /srv/zero2prod/etc/zero2prod config
  fi
}

_main "$@"

