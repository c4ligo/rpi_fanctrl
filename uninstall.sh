#!/bin/bash
# uninstall.sh

#Default variables and directories
INSTALL_DIR="/usr/local/bin/rpi_fanctrl"
INSTALL_PATH="$INSTALL_DIR/rpi_fanctrl"
ENV_PATH="$INSTALL_DIR/.env"
SERVICE_PATH="/etc/systemd/system/rpi_fanctrl.service"
SCRIPT_DIR=$(dirname "$(realpath "$0")")
WRAPPER_SCRIPT="/usr/local/bin/fanctrl"
SCRIPT_DIR_REMOVED="true"
# Updated variables and directories by installer script will be inserted below


# Stop the service before uninstalling.
echo "Stopping the service..."
sudo systemctl stop rpi_fanctrl.service


# Disable the service.
echo "Disabling the service..."
sudo systemctl disable rpi_fanctrl.service


# Remove the systemd service file.
echo "Removing systemd service file..."
sudo rm -f "$SERVICE_PATH"


# Remove the installed binary.
echo "Removing the program from $INSTALL_PATH..."
sudo rm -f "$INSTALL_PATH"


# Remove the .env file.
echo "Removing the .env file..."
sudo rm -f "$ENV_PATH"

# Remove the wrapper script from /usr/local/bin
if [ -f "$WRAPPER_SCRIPT" ]; then
    echo "Removing wrapper script at $WRAPPER_SCRIPT..."
    sudo rm -f "$WRAPPER_SCRIPT"
else
    echo "Wrapper script $WRAPPER_SCRIPT not found or already removed."
fi

# Remove the install directory and all other files.
if [ -d "$INSTALL_DIR" ] && [ "$(ls -A $INSTALL_DIR)" ]; then
    echo "Directory $INSTALL_DIR is not empty."
    read -p "Do you want to remove $INSTALL_DIR and all its contents (such as logs and statistics)? (y/n): " user_input

    if [[ "$user_input" =~ ^[Yy]$ ]]; then
        echo "Removing $INSTALL_DIR and all its contents..."
        sudo rm -rf "$INSTALL_DIR"
        echo "$INSTALL_DIR and its contents have been removed."
    else
        echo "Directory $INSTALL_DIR was not removed."
    fi
else
    echo "Directory $INSTALL_DIR is empty, removing it..."
    sudo rmdir "$INSTALL_DIR"
    echo "Empty directory $INSTALL_DIR has been removed."
fi

# Remove the initial directory with all included files if it was not removed in course of installation.
if [ "$SCRIPT_DIR_REMOVED" = "false" ]; then
    if [ -d "$SCRIPT_DIR" ] && [ "$(ls -A $SCRIPT_DIR)" ]; then
        echo "Directory $SCRIPT_DIR is not empty."
        read -p "Do you want to remove $SCRIPT_DIR and all its contents? (y/n): " user_input

        if [[ "$user_input" =~ ^[Yy]$ ]]; then
            echo "Removing $SCRIPT_DIR and all its contents..."
            sudo rm -rf "$SCRIPT_DIR"
            echo "$SCRIPT_DIR and its contents have been removed."
        else
            echo "Directory $SCRIPT_DIR was not removed."
        fi
    else
        echo "Directory $SCRIPT_DIR is empty, removing it..."
        sudo rmdir "$SCRIPT_DIR"
        echo "Empty directory $SCRIPT_DIR has been removed."
    fi
fi

# Reload systemd to reflect the changes.
echo "Reloading systemd..."
sudo systemctl daemon-reload

echo "Uninstallation complete."

# Keep terminal open
read -p "Press any key to exit..."