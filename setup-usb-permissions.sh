#!/bin/bash
# Setup USB permissions for Sound BlasterX G6
# This creates a udev rule so the app can access the device without sudo

echo "Setting up USB permissions for Sound BlasterX G6..."

# Create udev rule
sudo tee /etc/udev/rules.d/99-soundblaster-g6.rules > /dev/null << 'EOF'
# Sound BlasterX G6 - VID: 041e, PID: 3256
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="041e", ATTRS{idProduct}=="3256", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="usb", ATTRS{idVendor}=="041e", ATTRS{idProduct}=="3256", MODE="0666", TAG+="uaccess"
EOF

echo "✓ Udev rule created at /etc/udev/rules.d/99-soundblaster-g6.rules"

# Reload udev rules
echo "Reloading udev rules..."
sudo udevadm control --reload-rules
sudo udevadm trigger

echo ""
echo "✓ Setup complete!"
echo ""
echo "Please unplug and replug your Sound BlasterX G6 device."
echo "After that, the app will work without needing sudo."
