#!/bin/bash
set -e

# Phloem Unified Installation Script
# Handles OS detection, Ollama installation, and Phloem setup

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${PURPLE}[STEP]${NC} $1"
}

# Configuration
PHLOEM_VERSION="latest"
PHLOEM_DIR="$HOME/.phloem"
GITHUB_REPO="phloem-sh/phloem"
INSTALL_DIR="/usr/local/bin"
OLLAMA_MODEL="gemma3n:e2b"

# Global variables
OS=""
ARCH=""
PLATFORM=""

# Display banner
show_banner() {
    echo -e "${PURPLE}"
    cat << 'EOF'
  ____  _     _                       
 |  _ \| |__ | | ___   ___ _ __ ___  
 | |_) | '_ \| |/ _ \ / _ \ '_ ` _ \ 
 |  __/| | | | | (_) |  __/ | | | | |
 |_|   |_| |_|_|\___/ \___|_| |_| |_|
         Secure, fast command suggestions using local models
EOF
    echo -e "${NC}"
    echo "Installing Phloem with Ollama integration..."
    echo ""
}

# Detect OS and architecture
detect_platform() {
    log_step "Detecting platform..."
    
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    case "$OS" in
        "darwin")
            OS="macos"
            ;;
        "linux")
            OS="linux"
            ;;
        *)
            log_error "Unsupported operating system: $OS"
            exit 1
            ;;
    esac
    
    case "$ARCH" in
        "x86_64"|"amd64")
            ARCH="x86_64"
            ;;
        "arm64"|"aarch64")
            ARCH="aarch64"
            ;;
        *)
            log_error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac
    
    # Set platform for binary download
    case "$OS-$ARCH" in
        "macos-x86_64")
            PLATFORM="x86_64-apple-darwin"
            ;;
        "macos-aarch64")
            PLATFORM="aarch64-apple-darwin"
            ;;
        "linux-x86_64")
            PLATFORM="x86_64-unknown-linux-gnu"
            ;;
        "linux-aarch64")
            PLATFORM="aarch64-unknown-linux-gnu"
            ;;
        *)
            log_error "Unsupported platform combination: $OS-$ARCH"
            exit 1
            ;;
    esac
    
    log_success "Platform detected: $OS ($ARCH) -> $PLATFORM"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    log_step "Checking prerequisites..."
    
    # Check curl
    if ! command_exists curl; then
        log_error "curl is required but not installed"
        
        case "$OS" in
            "macos")
                log_info "Install curl with: brew install curl"
                ;;
            "linux")
                log_info "Install curl with: sudo apt-get install curl (Ubuntu/Debian) or sudo yum install curl (RHEL/CentOS)"
                ;;
        esac
        exit 1
    fi
    
    # Check if we can write to install directory
    if [ ! -w "$(dirname "$INSTALL_DIR")" ] && [ "$EUID" -ne 0 ]; then
        log_warning "May need sudo permissions to install to $INSTALL_DIR"
    fi
    
    log_success "Prerequisites check passed"
}

# Install Ollama
install_ollama() {
    log_step "Installing Ollama..."
    
    if command_exists ollama; then
        log_info "Ollama already installed: $(ollama --version 2>/dev/null || echo 'version unknown')"
        return 0
    fi
    
    log_info "Ollama not found, installing..."
    
    case "$OS" in
        "macos")
            # Try Homebrew first, then direct install
            if command_exists brew; then
                log_info "Installing Ollama via Homebrew..."
                brew install ollama || {
                    log_warning "Homebrew install failed, trying direct install..."
                    curl -fsSL https://ollama.ai/install.sh | sh
                }
            else
                log_info "Installing Ollama directly..."
                curl -fsSL https://ollama.ai/install.sh | sh
            fi
            ;;
        "linux")
            log_info "Installing Ollama for Linux..."
            curl -fsSL https://ollama.ai/install.sh | sh
            ;;
    esac
    
    # Verify installation
    if ! command_exists ollama; then
        log_error "Ollama installation failed"
        exit 1
    fi
    
    log_success "Ollama installed successfully"
}

# Start Ollama service
start_ollama_service() {
    log_step "Starting Ollama service..."
    
    # Check if already running
    if curl -s http://localhost:11434/api/version >/dev/null 2>&1; then
        log_info "Ollama service already running"
        return 0
    fi
    
    log_info "Starting Ollama service..."
    
    case "$OS" in
        "macos")
            # Start Ollama in the background
            nohup ollama serve >/dev/null 2>&1 &
            ;;
        "linux")
            # Try systemd first, then manual start
            if command_exists systemctl; then
                sudo systemctl start ollama || {
                    log_info "Systemd start failed, starting manually..."
                    nohup ollama serve >/dev/null 2>&1 &
                }
            else
                nohup ollama serve >/dev/null 2>&1 &
            fi
            ;;
    esac
    
    # Wait for service to start
    log_info "Waiting for Ollama service to start..."
    for i in {1..10}; do
        if curl -s http://localhost:11434/api/version >/dev/null 2>&1; then
            log_success "Ollama service started"
            return 0
        fi
        sleep 2
    done
    
    log_warning "Ollama service may not have started properly"
    log_info "You can start it manually with: ollama serve"
}

# Pull the Gemma model
pull_model() {
    log_step "Pulling Gemma model ($OLLAMA_MODEL)..."
    
    # Check if model already exists
    if ollama list | grep -q "$OLLAMA_MODEL"; then
        log_info "Model $OLLAMA_MODEL already available"
        return 0
    fi
    
    log_info "Pulling model $OLLAMA_MODEL (this may take several minutes)..."
    
    # Pull with progress indication
    ollama pull "$OLLAMA_MODEL" || {
        log_error "Failed to pull model $OLLAMA_MODEL"
        log_info "You can try pulling it manually later with: ollama pull $OLLAMA_MODEL"
        return 1
    }
    
    log_success "Model $OLLAMA_MODEL pulled successfully"
}

# Download and install Phloem binary
install_phloem_binary() {
    log_step "Installing Phloem binary..."
    
    # Check if already installed
    if command_exists phloem; then
        local current_version
        current_version=$(phloem --version 2>/dev/null | head -n1 || echo "unknown")
        log_info "Phloem already installed: $current_version"
        
        read -p "Do you want to reinstall? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Skipping binary installation"
            return 0
        fi
    fi
    
    # Create install directory if needed
    if [ ! -d "$INSTALL_DIR" ]; then
        log_info "Creating install directory: $INSTALL_DIR"
        sudo mkdir -p "$INSTALL_DIR" || {
            log_error "Failed to create install directory"
            exit 1
        }
    fi
    
    # Determine download URL
    local binary_url="https://github.com/$GITHUB_REPO/releases/$PHLOEM_VERSION/download/phloem-$PLATFORM"
    local temp_file
    temp_file=$(mktemp)
    
    log_info "Downloading Phloem binary from GitHub..."
    log_info "URL: $binary_url"
    
    if ! curl -L --fail --progress-bar "$binary_url" -o "$temp_file"; then
        log_error "Failed to download Phloem binary"
        log_info "Please check if the release exists at: https://github.com/$GITHUB_REPO/releases"
        rm -f "$temp_file"
        exit 1
    fi
    
    # Install binary
    log_info "Installing binary to $INSTALL_DIR/phloem"
    
    if [ -w "$INSTALL_DIR" ]; then
        mv "$temp_file" "$INSTALL_DIR/phloem"
        chmod +x "$INSTALL_DIR/phloem"
    else
        sudo mv "$temp_file" "$INSTALL_DIR/phloem"
        sudo chmod +x "$INSTALL_DIR/phloem"
    fi
    
    # Verify installation
    if ! "$INSTALL_DIR/phloem" --version >/dev/null 2>&1; then
        log_error "Binary installation verification failed"
        exit 1
    fi
    
    log_success "Phloem binary installed successfully"
}

# Initialize Phloem
initialize_phloem() {
    log_step "Initializing Phloem..."
    
    # Run phloem init
    if command_exists phloem; then
        phloem init || {
            log_warning "Phloem initialization failed, but continuing..."
        }
        log_success "Phloem initialized"
    else
        log_warning "Phloem binary not found in PATH, skipping initialization"
        log_info "You may need to add $INSTALL_DIR to your PATH"
    fi
}

# Setup shell integration
setup_shell_integration() {
    log_step "Setting up shell integration..."
    
    # Detect shell
    local shell_name
    shell_name=$(basename "$SHELL" 2>/dev/null || echo "bash")
    
    local rc_file=""
    case "$shell_name" in
        "bash")
            if [[ "$OS" == "macos" ]]; then
                rc_file="$HOME/.bash_profile"
            else
                rc_file="$HOME/.bashrc"
            fi
            ;;
        "zsh")
            rc_file="$HOME/.zshrc"
            ;;
        "fish")
            rc_file="$HOME/.config/fish/config.fish"
            mkdir -p "$(dirname "$rc_file")"
            ;;
        *)
            log_warning "Shell integration not available for $shell_name"
            return 0
            ;;
    esac
    
    # Check if PATH update is needed
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        log_info "Adding $INSTALL_DIR to PATH in $rc_file"
        
        # Add PATH export
        echo "" >> "$rc_file"
        echo "# Phloem" >> "$rc_file"
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$rc_file"
        
        log_success "Shell integration added to $rc_file"
        log_warning "Please restart your shell or run: source $rc_file"
    else
        log_info "$INSTALL_DIR already in PATH"
    fi
}

# Run health check
health_check() {
    log_step "Running health check..."
    
    local issues=0
    
    # Check binary
    if ! command_exists phloem; then
        log_error "❌ Phloem binary not found in PATH"
        issues=$((issues + 1))
    else
        log_success "✅ Phloem binary accessible"
    fi
    
    # Check Ollama service
    if ! curl -s http://localhost:11434/api/version >/dev/null 2>&1; then
        log_error "❌ Ollama service not running"
        log_info "Start it with: ollama serve"
        issues=$((issues + 1))
    else
        log_success "✅ Ollama service running"
    fi
    
    # Check model
    if ! ollama list 2>/dev/null | grep -q "$OLLAMA_MODEL"; then
        log_error "❌ Model $OLLAMA_MODEL not available"
        log_info "Pull it with: ollama pull $OLLAMA_MODEL"
        issues=$((issues + 1))
    else
        log_success "✅ Model $OLLAMA_MODEL available"
    fi
    
    # Check directory
    if [ ! -d "$PHLOEM_DIR" ]; then
        log_warning "⚠️  Phloem directory not initialized"
        log_info "Run: phloem init"
    else
        log_success "✅ Phloem directory exists"
    fi
    
    if [ $issues -eq 0 ]; then
        log_success "All health checks passed!"
    else
        log_warning "Found $issues issue(s) that may need attention"
    fi
    
    return $issues
}

# Show completion message
show_completion() {
    echo ""
    log_success "🎉 Phloem installation complete!"
    echo ""
    echo -e "${BLUE}Quick Start:${NC}"
    echo "  phloem \"list running processes\""
    echo "  phloem \"find large files\""
    echo "  phloem --explain \"git commit with message\""
    echo ""
    echo -e "${BLUE}Useful Commands:${NC}"
    echo "  phloem doctor          # Check system health"
    echo "  phloem --help          # Show help"
    echo "  ollama list              # List available models"
    echo "  ollama serve             # Start Ollama service"
    echo ""
    
    if ! command_exists phloem; then
        echo -e "${YELLOW}Note:${NC} You may need to restart your shell or run:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
    fi
    
    echo -e "${PURPLE}Documentation:${NC} https://github.com/$GITHUB_REPO"
    echo -e "${PURPLE}Issues:${NC} https://github.com/$GITHUB_REPO/issues"
}

# Cleanup on error
cleanup() {
    if [ $? -ne 0 ]; then
        log_error "Installation failed!"
        log_info "You can report issues at: https://github.com/$GITHUB_REPO/issues"
    fi
}

# Main installation function
main() {
    # Set up error handling
    trap cleanup EXIT
    
    show_banner
    detect_platform
    check_prerequisites
    install_ollama
    start_ollama_service
    pull_model
    install_phloem_binary
    initialize_phloem
    setup_shell_integration
    
    echo ""
    if health_check; then
        show_completion
    else
        log_warning "Installation completed with some issues"
        log_info "Run 'phloem doctor' for more details"
        show_completion
    fi
}

# Run main function
main "$@"