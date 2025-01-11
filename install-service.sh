#/bin/bash

set -e  # Exit on any command failure

# Copy service file to init.d
echo "Copying service file to /etc/init.d..."
cp ./cputemp-mqtt-rust-service /etc/init.d/cputemp-mqtt-rust-service

# Copy binary to /usr/sbin
echo "Copying binary to /usr/sbin..."
cp ./releases/cputemp-rust /usr/sbin/cputemp-mqtt-rust

# Set executable permissions for the service script
echo "Setting executable permissions for the service script..."
chmod +x /etc/init.d/cputemp-mqtt-rust-service

# Add the service to the default runlevel
echo "Adding service to the default runlevel..."
rc-update add cputemp-mqtt-rust-service default

# Start the service
echo "Starting the service..."
service cputemp-mqtt-rust-service start

echo "Service successfully installed and started."
