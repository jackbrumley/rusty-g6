Name:           rusty-g6
Version:        0.1.0
Release:        1%{?dist}
Summary:        Cross-platform GUI for SoundBlaster X G6 control

License:        MIT
URL:            https://github.com/jackbrumley/rusty-g6
Source0:        %{name}-%{version}.tar.gz

Requires:       libusb1

%description
Rusty G6 is a modern, user-friendly graphical interface for controlling
SoundBlaster X G6 audio device features on Linux and Windows.

Features include output control, surround sound, crystalizer, bass boost,
smart volume, dialog plus, and microphone configuration for Linux.

%prep
%setup -q

%build
# Binary is pre-built by Tauri

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 rusty-g6 %{buildroot}%{_bindir}/rusty-g6

%post
# Create udev rule for SoundBlaster X G6 USB access
echo 'SUBSYSTEM=="usb", ATTRS{idVendor}=="041e", ATTRS{idProduct}=="3256", TAG+="uaccess"' > /etc/udev/rules.d/50-soundblaster-x-g6.rules

# Reload udev rules
udevadm control --reload-rules
udevadm trigger

echo "Rusty G6 installed successfully!"
echo "USB permissions configured for SoundBlaster X G6"

%postun
# Remove udev rule on uninstall
if [ $1 -eq 0 ]; then
    rm -f /etc/udev/rules.d/50-soundblaster-x-g6.rules
    udevadm control --reload-rules
    udevadm trigger
fi

%files
%{_bindir}/rusty-g6

%changelog
* Sat Nov 30 2024 Jack Brumley <your-email@example.com> - 0.1.0-1
- Initial release
