#!/bin/sh
set -e

# Let the app open the Sound BlasterX G6 over USB without root.
mkdir -p /etc/udev/rules.d
cat > /etc/udev/rules.d/50-soundblaster-x-g6.rules << 'RULES'
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="041e", ATTRS{idProduct}=="3256", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="usb", ATTRS{idVendor}=="041e", ATTRS{idProduct}=="3256", MODE="0666", TAG+="uaccess"
RULES

# Reloading can fail where udev isn't running (containers, chroots); the rule
# still applies on the next boot, so don't fail the install over it.
udevadm control --reload-rules 2>/dev/null || true
udevadm trigger 2>/dev/null || true

exit 0
