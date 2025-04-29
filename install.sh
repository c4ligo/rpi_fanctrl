#!/bin/bash
# install.sh


# Automatically determine the directory where the script is located
SCRIPT_DIR=$(dirname "$(realpath "$0")")


# Set default variables and directories
INSTALL_DIR="/usr/local/bin/rpi_fanctrl"
INSTALL_PATH="$INSTALL_DIR/rpi_fanctrl"
ENV_PATH="$INSTALL_DIR/.env"
SERVICE_DIR="/etc/systemd/system"
SERVICE_PATH="$SERVICE_DIR/rpi_fanctrl.service"
WRAPPER_SCRIPT_NAME="fanctrl"
WRAPPER_DIR="/usr/local/bin"
WRAPPER_DEST="$WRAPPER_DIR/$WRAPPER_SCRIPT_NAME"


# Check if Rust is installed.
if ! command -v rustc &> /dev/null; then
    echo "Rust is not installed. Installing now..."

    # Ensure curl is installed.
    if ! command -v curl &> /dev/null; then
        echo "curl is not installed. Installing curl..."
        sudo apt update && sudo apt install -y curl
    fi

    # Install Rust for the current user.
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

    # Load Rust environment in the current session.
    source "$HOME/.cargo/env"

    echo "Rust installation complete."
else
    echo "Rust is already installed."
fi
echo ""


# Compile the Rust program.
echo "Compiling the Raspberry Pi Fan Control..."
cargo build --release
echo ""


# Prompt user to confirm or change installation directory
echo "The rpi_fanctrl binary is installed to a system-wide location in order to function properly as a systemd service."
echo "Default installation directory is: $INSTALL_DIR"
read -p "Do you want to use this location? (y/n): " yn
case "$yn" in
    [Yy]* )
        echo "Using default install location."
        ;;
    * )
        read -p "Enter the full system-wide path where you want to install the binary: " custom_path_install
        # Check if user entered something
        if [ -n "$custom_path_install" ]; then
            INSTALL_DIR="$custom_path_install"
            echo "Installation directory updated to: $INSTALL_DIR"
        else
            echo "No custom path entered. Keeping default: $INSTALL_DIR"
        fi
        ;;
esac


# Update Variables
INSTALL_PATH="$INSTALL_DIR/rpi_fanctrl"
ENV_PATH="$INSTALL_DIR/.env"
STAT_PATH="$INSTALL_DIR/stats.csv"
LOG_PATH="$INSTALL_DIR/event.log"
UNINSTALL_SCRIPT="$INSTALL_DIR/uninstall.sh"

# Update Bash wrapper script with correct uninstall script directory



# Write updated variables to the bash wrapper script
ENV_PATH_ADD="ENV_PATH='$ENV_PATH'"
sed -i '11i\'"$ENV_PATH_ADD" ./fanctrl
STAT_PATH_ADD="STAT_PATH='$STAT_PATH'"
sed -i '11i\'"$STAT_PATH_ADD" ./fanctrl
LOG_PATH_ADD="LOG_PATH='$LOG_PATH'"
sed -i '11i\'"$LOG_PATH_ADD" ./fanctrl
INSTALL_PATH_ADD="INSTALL_PATH='$INSTALL_PATH'"
sed -i '13i\'"$INSTALL_PATH_ADD" ./fanctrl
UNINSTALL_SCRIPT_DIR_ADD="UNINSTALL_SCRIPT='$UNINSTALL_SCRIPT'"
sed -i '11i\'"$UNINSTALL_SCRIPT_DIR_ADD" ./fanctrl


# Write updated variables to uninstall script
ENV_PATH_ADD="ENV_PATH='$ENV_PATH'"
sed -i '13i\'"$ENV_PATH_ADD" ./uninstall.sh
INSTALL_PATH_ADD="INSTALL_PATH='$INSTALL_PATH'"
sed -i '13i\'"$INSTALL_PATH_ADD" ./uninstall.sh
INSTALL_DIR_ADD="INSTALL_DIR='$INSTALL_DIR'"
sed -i '13i\'"$INSTALL_DIR_ADD" ./uninstall.sh
echo ""


# Move the compiled binary to a system-wide location and set permissions.
echo "Installing the program to $INSTALL_PATH..."
sudo mkdir -p "$INSTALL_DIR"
# 4755 grants root permissions, required for the modification of GPIO pins.
sudo install -m 4755 target/release/rpi_fanctrl "$INSTALL_PATH"
echo ""


# Create the .env file with default values.
echo "The .env file is created in the same folder as the binary: $INSTALL_DIR"
echo "If you record logs and statistics, these will be also saved in the same folder."
echo "Creating .env file with default values..."
sudo tee "$ENV_PATH" > /dev/null <<EOF
gpio_pin=18
off_temp=45.0
min_temp=48.0
max_temp=70.0
min_duty_cycle=0.20
max_duty_cycle=1.00
pwm_freq=1000
delay_hot=1.0
delay_cold=10.0
delay_time=5000
error_time=10
log_option=true
temp_freq=10
stat_option=true
stat_freq=10
stat_min_time=10
stat_max_time=6000
stat_delta_t=5.0
EOF

echo ".env file created at $ENV_PATH"
echo ""


# Open .env file.            
read -p "Do you want to open the .env file to view or edit the environment variables? (y/n): " yn
case $yn in
    [Yy]* )
        sudo ${EDITOR:-nano} "$ENV_PATH"
        ;;
    * )
        echo "Skipped opening .env file."
        ;;
esac
echo ""


# Prompt user to confirm or change the directory of the systemd service files
echo "The rpi_fanctrl systemd service file is created in a system-wide location in order to function properly."
echo "Default installation directory is: $SERVICE_DIR"
read -p "Do you want to use this location? (y/n): " yn
case "$yn" in
    [Yy]* )
        echo "Using default install location."
        ;;
    * )
        read -p "Enter the full system-wide path where you want to create the systemd service file: " custom_path_service
        # Check if user entered something
        if [ -n "$custom_path_service" ]; then
            SERVICE_DIR="$custom_path_service"
            echo "systemd service file directory updated to: $SERVICE_DIR"
        else
            echo "No custom path entered. Keeping default: $SERVICE_DIR"
        fi
        ;;
esac


# Update Variables
SERVICE_PATH="$SERVICE_DIR/rpi_fanctrl.service"


# Write updated variables to the bash wrapper script
SERVICE_PATH_ADD="SERVICE_PATH='$SERVICE_PATH'"
sed -i '11i\'"$SERVICE_PATH_ADD" ./fanctrl


# Write updated variables to uninstall script
SERVICE_PATH_ADD="SERVICE_PATH='$SERVICE_PATH'"
sed -i '13i\'"$SERVICE_PATH_ADD" ./uninstall.sh
echo ""


# Create systemd service file for auto-start.
echo "Creating systemd service file..."
sudo tee "$SERVICE_PATH" > /dev/null <<EOF
[Unit]
Description=Raspberry Pi Fan Control
After=multi-user.target

[Service]
ExecStart=$INSTALL_PATH
WorkingDirectory=$INSTALL_DIR
Restart=on-failure
RestartSec=10
User=root

[Install]
WantedBy=multi-user.target
EOF

echo "systemd service file created at $SERVICE_PATH"
echo ""


# Open systemd service file.            
read -p "Do you want to open the systemd service file to view or edit it? (y/n): " yn
case $yn in
    [Yy]* )
        sudo ${EDITOR:-nano} "$SERVICE_PATH"
        ;;
    * )
        echo "Skipped opening the systemd service file."
        ;;
esac
echo ""


# Prompt user to confirm or change the directory of the Bash wrapper script
echo "A Bash wrapper script is installed to allow easy control of the rpi_fanctrl program."
echo "The script is added to the PATH for convenient execution."
echo "Default PATH location is: $WRAPPER_DIR"
read -p "Do you want to use this location? (y/n): " yn
case "$yn" in
    [Yy]* )
        echo "Using default PATH location."
        ;;
    * )
        read -p "Enter the full system-wide path where you want to create the Bash wrapper script: " custom_path_wrapper
        # Check if user entered something
        if [ -n "$custom_path_wrapper" ]; then
            WRAPPER_DIR="$custom_path_wrapper"
            echo "PATH location updated to: $WRAPPER_DIR"
        else
            echo "No custom PATH location entered. Keeping default: $WRAPPER_DIR"
        fi
        ;;
esac


# Update variables
WRAPPER_DEST="$WRAPPER_DIR/$WRAPPER_SCRIPT_NAME"


# Write updated variables to the Bash wrapper script
WRAPPER_SCRIPT_ADD="WRAPPER_SCRIPT='$WRAPPER_DEST'"
sed -i '11i\'"$WRAPPER_SCRIPT_ADD" ./fanctrl
echo ""


# Write updated variables to uninstall script
WRAPPER_SCRIPT_ADD="WRAPPER_SCRIPT='$WRAPPER_DEST'"
sed -i '13i\'"$WRAPPER_SCRIPT_ADD" ./uninstall.sh
echo ""


# Install the Bash wrapper script to control the service
echo "Installing Bash wrapper script to $WRAPPER_DEST..."
chmod +x "$WRAPPER_SCRIPT_NAME"
sudo cp "$WRAPPER_SCRIPT_NAME" "$WRAPPER_DEST"
echo ""


# Check if wrapper directory is actually in PATH
if [[ ":$PATH:" != *":$WRAPPER_DIR:"* ]]; then
    echo "WARNING: $WRAPPER_DIR is not in your PATH."
    echo "To make the 'fanctrl' command available, add this line to your shell config (e.g. ~/.bashrc):"
    echo "export PATH=\"$WRAPPER_DIR:\$PATH\""
    echo "Then run: source ~/.bashrc"
    echo ""
fi


# Ask user if they want to remove the initial folder
read -p "Do you want the installer to remove the initial folder: $SCRIPT_DIR ? (y/n): " yn
case $yn in
    [Yy]* )
        # Update the uninstall script
        SCRIPT_DIR_ADD="SCRIPT_DIR='$SCRIPT_DIR'"
        sed -i '13i\'"$SCRIPT_DIR_ADD" ./uninstall.sh
        SCRIPT_DIR_REMOVED="SCRIPT_DIR_REMOVED='true'"
        sed -i '13i\'"$SCRIPT_DIR_REMOVED" ./uninstall.sh
        REMOVE_INIT_DIR="true"
        echo ""        
        ;;
    * )
        # Update the uninstall script
        SCRIPT_DIR_ADD="SCRIPT_DIR='$SCRIPT_DIR'"
        sed -i '13i\'"$SCRIPT_DIR_ADD" ./uninstall.sh
        SCRIPT_DIR_REMOVED="SCRIPT_DIR_REMOVED='false'"
        REMOVE_INIT_DIR="false"
        sed -i '13i\'"$SCRIPT_DIR_REMOVED" ./uninstall.sh
        echo "Initial folder will not be deleted."
        echo ""
        ;;
esac


# Move the uninstall script to the install directory and make it executable
echo "Moving uninstall script to $INSTALL_DIR..."
sudo mv uninstall.sh "$UNINSTALL_SCRIPT"
sudo chmod +x "$UNINSTALL_SCRIPT"


# Remove the initial directory with all included files
if [ "$REMOVE_INIT_DIR" = "true" ]; then
    echo "Removing $SCRIPT_DIR and all its contents..."
    sudo rm -rf "$SCRIPT_DIR"
    echo "$SCRIPT_DIR and its contents have been removed."
    echo ""
fi


# Reload systemd and enable the service.
echo "Reloading systemd and enabling the service..."
sudo systemctl daemon-reload
sudo systemctl enable rpi_fanctrl.service


# Start the service.
echo "Starting the service..."
sudo systemctl start rpi_fanctrl.service


# Verify that the service is running.
echo "Verifying that the service is running..."
sudo systemctl status rpi_fanctrl.service
echo ""


# Installation directories
echo "Installation directories:"
echo "Program: $INSTALL_PATH"
echo "Systemd service file: $SERVICE_PATH"
echo "Environmental variables: $ENV_PATH"
echo "Statistics log (if enabled): $STAT_PATH"
echo "Event log (if enabled): $LOG_PATH"
echo "Wrapper script: $WRAPPER_DEST"
echo "Uninstall script: $UNINSTALL_SCRIPT"


# Installation complete
echo "Installation complete."
echo "Wrapper command available as: sudo fanctrl"
echo "Systemd service created and started."
echo ""


# Keep terminal open
read -p "Press any key to exit..."