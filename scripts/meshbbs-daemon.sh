#!/usr/bin/env bash
# meshbbs daemon control script
# Cross-platform helper for starting, stopping, and managing meshbbs daemon
# Works on Linux and macOS

set -euo pipefail

# Configuration (can be overridden with environment variables)
: "${MESHBBS_BIN:=./target/release/meshbbs}"
: "${MESHBBS_CONFIG:=config.toml}"
: "${MESHBBS_PID_FILE:=/tmp/meshbbs.pid}"
: "${MESHBBS_PORT:=}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Check if daemon is running
is_running() {
    if [[ ! -f "$MESHBBS_PID_FILE" ]]; then
        return 1
    fi
    
    local pid
    pid=$(cat "$MESHBBS_PID_FILE" 2>/dev/null || echo "")
    
    if [[ -z "$pid" ]]; then
        return 1
    fi
    
    # Check if process is actually running
    if kill -0 "$pid" 2>/dev/null; then
        return 0
    else
        # PID file exists but process is dead
        rm -f "$MESHBBS_PID_FILE"
        return 1
    fi
}

# Get PID if running
get_pid() {
    if [[ -f "$MESHBBS_PID_FILE" ]]; then
        cat "$MESHBBS_PID_FILE"
    else
        echo ""
    fi
}

# Start daemon
start() {
    if is_running; then
        print_error "meshbbs is already running (PID: $(get_pid))"
        return 1
    fi
    
    if [[ ! -f "$MESHBBS_BIN" ]]; then
        print_error "meshbbs binary not found at: $MESHBBS_BIN"
        print_warning "Build with: cargo build --release --features daemon"
        return 1
    fi
    
    if [[ ! -f "$MESHBBS_CONFIG" ]]; then
        print_error "Config file not found at: $MESHBBS_CONFIG"
        print_warning "Initialize with: $MESHBBS_BIN init"
        return 1
    fi
    
    print_status "Starting meshbbs daemon..."
    
    # Build command
    local cmd=("$MESHBBS_BIN" "--config" "$MESHBBS_CONFIG" "start" "--daemon" "--pid-file" "$MESHBBS_PID_FILE")
    
    if [[ -n "$MESHBBS_PORT" ]]; then
        cmd+=("--port" "$MESHBBS_PORT")
    fi
    
    # Start daemon
    "${cmd[@]}"
    
    # Wait a moment and check if it started
    sleep 1
    
    if is_running; then
        print_status "meshbbs started successfully (PID: $(get_pid))"
        return 0
    else
        print_error "meshbbs failed to start"
        print_warning "Check logs at: meshbbs.log"
        return 1
    fi
}

# Stop daemon
stop() {
    if ! is_running; then
        print_warning "meshbbs is not running"
        return 0
    fi
    
    local pid
    pid=$(get_pid)
    
    print_status "Stopping meshbbs (PID: $pid)..."
    
    # Send SIGTERM for graceful shutdown
    kill -TERM "$pid" 2>/dev/null || true
    
    # Wait for process to exit (max 10 seconds)
    local count=0
    while is_running && [[ $count -lt 20 ]]; do
        sleep 0.5
        ((count++))
    done
    
    if is_running; then
        print_warning "Process did not stop gracefully, sending SIGKILL..."
        kill -KILL "$pid" 2>/dev/null || true
        sleep 1
    fi
    
    # Clean up PID file
    rm -f "$MESHBBS_PID_FILE"
    
    print_status "meshbbs stopped"
    return 0
}

# Restart daemon
restart() {
    print_status "Restarting meshbbs..."
    stop
    sleep 1
    start
}

# Show status
status() {
    if is_running; then
        local pid
        pid=$(get_pid)
        print_status "meshbbs is running (PID: $pid)"
        
        # Show additional info if available
        if command -v ps &>/dev/null; then
            echo ""
            echo "Process details:"
            ps -p "$pid" -o pid,ppid,user,%cpu,%mem,etime,command 2>/dev/null || true
        fi
        
        return 0
    else
        print_warning "meshbbs is not running"
        return 1
    fi
}

# Show logs (tail)
logs() {
    local log_file="meshbbs.log"
    local lines="${1:-50}"
    
    if [[ ! -f "$log_file" ]]; then
        print_error "Log file not found: $log_file"
        return 1
    fi
    
    tail -n "$lines" -f "$log_file"
}

# Usage information
usage() {
    cat <<EOF
Usage: $0 {start|stop|restart|status|logs [lines]}

Meshbbs Daemon Control Script

Commands:
    start       Start the meshbbs daemon
    stop        Stop the meshbbs daemon (graceful SIGTERM)
    restart     Restart the meshbbs daemon
    status      Show daemon status and PID
    logs        Tail the log file (default: 50 lines)

Environment Variables:
    MESHBBS_BIN       Path to meshbbs binary (default: ./target/release/meshbbs)
    MESHBBS_CONFIG    Path to config file (default: config.toml)
    MESHBBS_PID_FILE  Path to PID file (default: /tmp/meshbbs.pid)
    MESHBBS_PORT      Meshtastic device port (optional)

Examples:
    # Start daemon
    $0 start
    
    # Start with custom port
    MESHBBS_PORT=/dev/ttyUSB0 $0 start
    
    # Check status
    $0 status
    
    # View logs
    $0 logs
    
    # Restart
    $0 restart

EOF
}

# Main command dispatch
main() {
    local command="${1:-}"
    
    case "$command" in
        start)
            start
            ;;
        stop)
            stop
            ;;
        restart)
            restart
            ;;
        status)
            status
            ;;
        logs)
            logs "${2:-50}"
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
