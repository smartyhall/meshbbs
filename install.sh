#!/bin/bash
# MeshBBS Installation Script
# Installs MeshBBS to /opt/meshbbs (or custom location)
# Usage: sudo ./install.sh [install_path]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default installation path
INSTALL_PATH="${1:-/opt/meshbbs}"
SERVICE_USER="${MESHBBS_USER:-bbs}"
SERVICE_GROUP="${MESHBBS_GROUP:-bbs}"

echo -e "${GREEN}MeshBBS Installation Script${NC}"
echo "========================================"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
    exit 1
fi

# Check if binary exists
if [ ! -f "target/release/meshbbs" ]; then
    echo -e "${RED}Error: Binary not found at target/release/meshbbs${NC}"
    echo "Please run 'cargo build --release' first"
    exit 1
fi

echo -e "${YELLOW}Installation Configuration:${NC}"
echo "  Install path: $INSTALL_PATH"
echo "  Service user: $SERVICE_USER"
echo "  Service group: $SERVICE_GROUP"
echo ""

# Confirm installation
read -p "Continue with installation? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Installation cancelled."
    exit 0
fi

echo ""
echo "Installing MeshBBS..."

# Create service user if it doesn't exist
if ! id "$SERVICE_USER" &>/dev/null; then
    echo -e "${YELLOW}Creating service user: $SERVICE_USER${NC}"
    useradd -r -s /bin/false -d "$INSTALL_PATH" -c "MeshBBS Service User" "$SERVICE_USER"
fi

# Add user to dialout group for serial port access
echo -e "${YELLOW}Adding $SERVICE_USER to dialout group${NC}"
usermod -a -G dialout "$SERVICE_USER" 2>/dev/null || true

# Create installation directory
echo -e "${YELLOW}Creating installation directory: $INSTALL_PATH${NC}"
mkdir -p "$INSTALL_PATH"
mkdir -p "$INSTALL_PATH/data"
mkdir -p "$INSTALL_PATH/data/messages"
mkdir -p "$INSTALL_PATH/data/users"
mkdir -p "$INSTALL_PATH/data/files"
mkdir -p "$INSTALL_PATH/data/tinymush"
mkdir -p "$INSTALL_PATH/data/backups"

# Copy binary
echo -e "${YELLOW}Installing binary${NC}"
cp target/release/meshbbs "$INSTALL_PATH/"
chmod 755 "$INSTALL_PATH/meshbbs"

# Copy or create configuration
if [ -f "config.toml" ] && [ ! -f "$INSTALL_PATH/config.toml" ]; then
    echo -e "${YELLOW}Installing existing config.toml${NC}"
    cp config.toml "$INSTALL_PATH/"
elif [ ! -f "$INSTALL_PATH/config.toml" ]; then
    echo -e "${YELLOW}Creating config.toml from example${NC}"
    if [ -f "config.example.toml" ]; then
        cp config.example.toml "$INSTALL_PATH/config.toml"
    else
        echo -e "${RED}Warning: No config file found. You'll need to create one.${NC}"
    fi
fi
chmod 644 "$INSTALL_PATH/config.toml" 2>/dev/null || true

# Copy topics example if data/topics.json doesn't exist
if [ ! -f "$INSTALL_PATH/data/topics.json" ]; then
    if [ -f "topics.example.json" ]; then
        echo -e "${YELLOW}Installing topics.json${NC}"
        cp topics.example.json "$INSTALL_PATH/data/topics.json"
    fi
fi

# Set ownership
echo -e "${YELLOW}Setting ownership${NC}"
chown -R "$SERVICE_USER:$SERVICE_GROUP" "$INSTALL_PATH"

# Set permissions
chmod 755 "$INSTALL_PATH"
chmod 755 "$INSTALL_PATH/data"
chmod 755 "$INSTALL_PATH/meshbbs"

# Install systemd service if systemd is available
if command -v systemctl &> /dev/null; then
    echo -e "${YELLOW}Installing systemd service${NC}"
    
    # Create service file
    cat > /etc/systemd/system/meshbbs.service <<EOF
[Unit]
Description=MeshBBS - Bulletin Board System for Meshtastic
Documentation=https://martinbogo.github.io/meshbbs/
After=network.target
Wants=network.target

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_GROUP
WorkingDirectory=$INSTALL_PATH
ExecStart=$INSTALL_PATH/meshbbs
Restart=always
RestartSec=10
TimeoutStopSec=30

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=meshbbs

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$INSTALL_PATH/data $INSTALL_PATH/meshbbs.log
ProtectKernelTunables=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictNamespaces=true

# Environment
Environment="RUST_LOG=info"

# Resource limits
MemoryLimit=512M
TasksMax=100

[Install]
WantedBy=multi-user.target
EOF

    # Reload systemd
    systemctl daemon-reload
    
    echo ""
    echo -e "${GREEN}Systemd service installed!${NC}"
    echo "To enable and start the service:"
    echo "  sudo systemctl enable meshbbs"
    echo "  sudo systemctl start meshbbs"
    echo ""
    echo "To check status:"
    echo "  sudo systemctl status meshbbs"
    echo "  sudo journalctl -u meshbbs -f"
fi

echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Installation details:"
echo "  Binary:        $INSTALL_PATH/meshbbs"
echo "  Config:        $INSTALL_PATH/config.toml"
echo "  Data:          $INSTALL_PATH/data"
echo "  User:          $SERVICE_USER"
echo ""
echo "Next steps:"
echo "  1. Edit configuration: sudo nano $INSTALL_PATH/config.toml"
echo "  2. Set sysop password in config.toml"
echo "  3. Configure serial port (usually /dev/ttyACM0 or /dev/ttyUSB0)"
if command -v systemctl &> /dev/null; then
    echo "  4. Enable service: sudo systemctl enable meshbbs"
    echo "  5. Start service: sudo systemctl start meshbbs"
else
    echo "  4. Run manually: cd $INSTALL_PATH && sudo -u $SERVICE_USER ./meshbbs"
fi
echo ""
echo "Documentation: $INSTALL_PATH/docs/ or https://martinbogo.github.io/meshbbs/"
echo ""
