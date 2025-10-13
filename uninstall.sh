#!/bin/bash
# MeshBBS Uninstall Script
# Removes MeshBBS from system
# Usage: sudo ./uninstall.sh [install_path]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default installation path
INSTALL_PATH="${1:-/opt/meshbbs}"
SERVICE_USER="${MESHBBS_USER:-bbs}"

echo -e "${RED}MeshBBS Uninstall Script${NC}"
echo "========================================"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
    exit 1
fi

# Check if installation exists
if [ ! -d "$INSTALL_PATH" ]; then
    echo -e "${YELLOW}Warning: Installation path $INSTALL_PATH does not exist${NC}"
    echo "Nothing to uninstall."
    exit 0
fi

echo -e "${RED}WARNING: This will remove:${NC}"
echo "  - Binary: $INSTALL_PATH/meshbbs"
echo "  - Data directory: $INSTALL_PATH/data"
echo "  - Configuration: $INSTALL_PATH/config.toml"
echo "  - Systemd service (if installed)"
echo ""
echo -e "${RED}ALL DATA WILL BE LOST!${NC}"
echo ""

# Confirm uninstallation
read -p "Are you sure you want to uninstall MeshBBS? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Uninstallation cancelled."
    exit 0
fi

echo ""
read -p "Create backup before uninstalling? (Y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Nn]$ ]]; then
    BACKUP_DIR="$HOME/meshbbs-backup-$(date +%Y%m%d-%H%M%S)"
    echo -e "${YELLOW}Creating backup at: $BACKUP_DIR${NC}"
    mkdir -p "$BACKUP_DIR"
    cp -r "$INSTALL_PATH/data" "$BACKUP_DIR/" 2>/dev/null || true
    cp "$INSTALL_PATH/config.toml" "$BACKUP_DIR/" 2>/dev/null || true
    echo -e "${GREEN}Backup created: $BACKUP_DIR${NC}"
    echo ""
fi

echo "Uninstalling MeshBBS..."

# Stop and disable systemd service if it exists
if command -v systemctl &> /dev/null && systemctl is-active --quiet meshbbs; then
    echo -e "${YELLOW}Stopping MeshBBS service${NC}"
    systemctl stop meshbbs
fi

if command -v systemctl &> /dev/null && systemctl is-enabled --quiet meshbbs; then
    echo -e "${YELLOW}Disabling MeshBBS service${NC}"
    systemctl disable meshbbs
fi

# Remove systemd service file
if [ -f /etc/systemd/system/meshbbs.service ]; then
    echo -e "${YELLOW}Removing systemd service file${NC}"
    rm -f /etc/systemd/system/meshbbs.service
    systemctl daemon-reload
fi

# Remove installation directory
echo -e "${YELLOW}Removing installation directory: $INSTALL_PATH${NC}"
rm -rf "$INSTALL_PATH"

# Ask about removing user
read -p "Remove service user '$SERVICE_USER'? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if id "$SERVICE_USER" &>/dev/null; then
        echo -e "${YELLOW}Removing user: $SERVICE_USER${NC}"
        userdel "$SERVICE_USER" 2>/dev/null || true
    fi
fi

echo ""
echo -e "${GREEN}Uninstallation complete!${NC}"
echo ""
if [ -n "$BACKUP_DIR" ]; then
    echo "Your data has been backed up to: $BACKUP_DIR"
fi
echo ""
