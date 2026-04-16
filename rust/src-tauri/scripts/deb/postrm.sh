#!/bin/sh
set -e

case "$1" in
  remove|purge)
    rm -f /etc/udev/rules.d/50-soundblaster-x-g6.rules
    udevadm control --reload-rules
    udevadm trigger
    ;;
esac

exit 0
