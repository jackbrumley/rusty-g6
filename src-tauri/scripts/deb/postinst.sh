#!/bin/sh
set -e

cat > /etc/udev/rules.d/50-soundblaster-x-g6.rules << 'EOF'
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="041e", ATTRS{idProduct}=="3256", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="usb", ATTRS{idVendor}=="041e", ATTRS{idProduct}=="3256", MODE="0666", TAG+="uaccess"
EOF

udevadm control --reload-rules
udevadm trigger

exit 0
