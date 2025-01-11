#/bin/bash

set -e  # Exit on any command failure

# Stop the service and wait for it to complete
echo "Stopping service..."
service cputemp-mqtt-rust-service stop

# Remove the service from default runlevel and wait for it to complete
echo "Removing service from default runlevel..."
rc-update del cputemp-mqtt-rust-service default

# Remove files
echo "Removing service binaries and configuration files..."
rm -f /usr/sbin/cputemp-mqtt-rust
rm -f /etc/init.d/cputemp-mqtt-rust-service
rm -f /var/log/cputemp-mqtt-rust-service.log

echo "Service successfully removed."
