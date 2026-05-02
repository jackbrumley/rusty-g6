#!/bin/sh
set -e

if [ "$1" -eq 0 ]; then
  rm -f /etc/udev/rules.d/50-soundblaster-x-g6.rules
  udevadm control --reload-rules
  udevadm trigger
fi

exit 0
