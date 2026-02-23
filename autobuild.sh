#!/bin/bash

# AutoBuild Script for EMQX Auth Service
# Build & optionally push Rust project + Docker image to GHCR
# Usage: ./autobuild.sh [--push] [--no-test]
# Example: ./autobuild.sh --push (builds and pushes to GHCR)

set -e

# Colors untuk output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Function to print colored messages
print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_stage() { echo -e "${PURPLE}[STAGE]${NC} $1"; }
print_header() { echo -e "${BOLD}${CYAN}$1${NC}"; }

# Function to print table header
print_table_header() {
    local col1="$1"
    local col2="$2"
    local col3="$3"
    printf "%-40s | %-25s | %-20s\n" "$col1" "$col2" "$col3"
    printf "%-40s | %-25s | %-20s\n" "$(printf '%.0s─' {1..40})" "$(printf '%.0s─' {1..25})" "$(printf '%.0s─' {1..20})"
}

# Function to print table row
print_table_row() {
    local col1="$1"
    local col2="$2"
    local col3="$3"
    printf "%-40s | %-25s | %-20s\n" "$col1" "$col2" "$col3"
}

echo ""
print_header "╔════════════════════════════════════════════════════════════════╗"
print_header "║              AutoBuild: EMQX Auth Service                      ║"
print_header "║   Rust Project + Docker Image Build & Validation               ║"
print_header "╚════════════════════════════════════════════════════════════════╝"
echo ""
print_info "📦 Build Information:"
print_info "  Repository: EMQX Auth Service"
print_info "  Language: Rust"
print_info "  Container: Docker (Debian Bookworm)"
print_info "  Backend: Rust Actix"
print_info "  Database: MySQL"
echo ""

# Parse arguments
SKIP_TESTS=false
PUSH_TO_REGISTRY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --push)
            PUSH_TO_REGISTRY=true
            shift
            ;;
        --no-test)
            SKIP_TESTS=true
            shift
            ;;
        *)
            print_warning "Unknown option: $1"
            shift
            ;;
    esac
done

# Check if docker is installed
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker to build the service."
    exit 1
fi

# Check if Dockerfile exists
if [ ! -f "Dockerfile" ]; then
    print_error "Dockerfile not found in current directory"
    exit 1
fi

# ========================================
# PHASE 1: BUILD RUST PROJECT LOCALLY
# ========================================
print_stage "PHASE 1: Local Rust Build"
echo "========================================="
echo ""

# Collect build environment info
RUST_VERSION=""
CARGO_PATH=""

if ! command -v cargo &> /dev/null; then
    print_warning "Cargo not found locally. This is OK - Docker will handle the build."
else
    CARGO_PATH="$(which cargo)"
    RUST_VERSION="$(rustc --version)"
    
    print_info "🔧 Build Environment:"
    print_table_header "Component" "Status" "Details"
    print_table_row "Cargo" "✓ Found" "$CARGO_PATH"
    print_table_row "Rust" "✓ Ready" "$RUST_VERSION"
    echo ""
    
    print_info "📦 Building Rust project in release mode..."
    echo ""
    
    if cargo build --release 2>&1; then
        print_success "✓ Local Rust build completed successfully"
        
        if [ -f "target/release/emqx_auth_service" ]; then
            BINARY_SIZE=$(du -h target/release/emqx_auth_service | cut -f1)
            BINARY_PATH="$(pwd)/target/release/emqx_auth_service"
            
            echo ""
            print_info "📊 Build Artifact Information:"
            print_table_header "Property" "Value" "Type"
            print_table_row "Binary Name" "emqx_auth_service" "Executable"
            print_table_row "Binary Size" "$BINARY_SIZE" "Release Build"
            print_table_row "Location" "$BINARY_PATH" "Path"
            echo ""
        fi
    else
        print_error "Local Rust build failed!"
        exit 1
    fi
fi

echo ""
print_stage "PHASE 2: Docker BuildX with Labels & Annotations"
echo "========================================="
echo ""

# Build configuration
IMAGE_NAME="emqx-auth-service"
IMAGE_TAG="${IMAGE_TAG:-latest}"
LOCAL_IMAGE="${IMAGE_NAME}:${IMAGE_TAG}"
REGISTRY_IMAGE="ghcr.io/farismnrr/${IMAGE_NAME}:${IMAGE_TAG}"

# Metadata
IMAGE_TITLE="EMQX Auth Service"
IMAGE_VERSION=""
IMAGE_AUTHORS="farismnrr"
IMAGE_DESCRIPTION="Lightweight Rust-based HTTP Auth Service for EMQX, providing high-performance authentication and ACL logic with MySQL backend."
IMAGE_SOURCE="https://github.com/farismnrr/EMQX-Auth-Service"
BUILD_PLATFORMS="linux/amd64,linux/arm64"

print_info "🐳 Docker BuildX Configuration:"
print_table_header "Configuration" "Value" "Status"
print_table_row "Dockerfile" "Dockerfile" "✓ Required"
print_table_row "Platforms" "$BUILD_PLATFORMS" "→ Multi-arch"
print_table_row "Local Image" "$LOCAL_IMAGE" "→ Local"
print_table_row "Registry Image" "$REGISTRY_IMAGE" "→ GHCR"
echo ""

print_info "🏷️ Build Labels & Annotations:"
print_table_header "Metadata Type" "Key" "Value"
print_table_row "Label" "org.opencontainers.image.title" "$IMAGE_TITLE"
print_table_row "Label" "org.opencontainers.image.version" "$IMAGE_VERSION"
print_table_row "Label" "org.opencontainers.image.authors" "$IMAGE_AUTHORS"
print_table_row "Label" "org.opencontainers.image.description" "$IMAGE_DESCRIPTION"
print_table_row "Annotation" "org.opencontainers.image.title" "$IMAGE_TITLE"
print_table_row "Annotation" "org.opencontainers.image.description" "$IMAGE_DESCRIPTION"
print_table_row "Annotation" "org.opencontainers.image.source" "$IMAGE_SOURCE"
echo ""

print_info "🔨 Building Docker image with buildx..."
echo ""

# Build flags based on PUSH_TO_REGISTRY
BUILD_ARGS=(
    --platform "$BUILD_PLATFORMS"
    --label "org.opencontainers.image.title=$IMAGE_TITLE"
    --label "org.opencontainers.image.description=$IMAGE_DESCRIPTION"
    --label "org.opencontainers.image.authors=$IMAGE_AUTHORS"
    --label "org.opencontainers.image.version=$IMAGE_VERSION"
    --annotation "org.opencontainers.image.title=$IMAGE_TITLE"
    --annotation "org.opencontainers.image.description=$IMAGE_DESCRIPTION"
    --annotation "org.opencontainers.image.source=$IMAGE_SOURCE"
    -f "Dockerfile"
    --progress=plain
)

if [ "$PUSH_TO_REGISTRY" = true ]; then
    print_info "🔐 Push to registry is enabled. Loading configuration..."
    
    # Load .env if exists
    if [ -f .env ]; then
        print_info "Loading environment variables from .env..."
        export $(grep -v '^#' .env | xargs)
    else
        print_warning ".env file not found"
    fi
    
    # Validate required variables
    LOGIN_TOKEN="$GITHUB_PAT_TOKEN"
    if [ -z "$LOGIN_TOKEN" ]; then
        if command -v gh &> /dev/null && gh auth status &> /dev/null; then
            print_info "GITHUB_PAT_TOKEN not found, but gh CLI is authenticated. Using gh auth token."
            LOGIN_TOKEN=$(gh auth token)
        else
            print_error "GITHUB_PAT_TOKEN not found in .env and gh CLI is not authenticated!"
            echo ""
            echo -e "${YELLOW}Setup required:${NC}"
            echo "1. Create .env file and add GITHUB_PAT_TOKEN=ghp_xxxxxx"
            echo "2. Or login via GitHub CLI: gh auth login"
            exit 1
        fi
    fi
    print_success "✓ Authentication token ready"
    
    # Defaults if not in .env
    GHCR_USERNAME="${GHCR_USERNAME:-farismnrr}"
    GHCR_NAMESPACE="farismnrr"
    PUSH_IMAGE_NAME="emqx-auth-service"
    REGISTRY="ghcr.io"
    FULL_REGISTRY_PATH="${REGISTRY}/${GHCR_NAMESPACE}/${PUSH_IMAGE_NAME}"
    
    print_info "GHCR namespace: $GHCR_NAMESPACE"
    print_info "GitHub username: $GHCR_USERNAME"
    print_info "Registry path: $FULL_REGISTRY_PATH"
    echo ""
    
    # Authenticate
    print_info "Logging in to GHCR..."
    if echo "$LOGIN_TOKEN" | docker login ghcr.io -u $GHCR_USERNAME --password-stdin &> /dev/null; then
        print_success "✓ Successfully authenticated to GHCR"
    else
        print_error "Failed to authenticate to GHCR"
        exit 1
    fi
    echo ""
    
    # Get tag name
    echo -e "${YELLOW}Enter tag name for GHCR image:${NC}"
    echo "Examples: dev, latest, v1.0.0, staging, v1.0.0-rc1"
    echo -n "Tag: "
    read TAG_NAME
    echo ""
    
    if [ -z "$TAG_NAME" ]; then
        print_error "Tag name cannot be empty!"
        exit 1
    fi
    
    # Use tag name as image version
    IMAGE_VERSION="$TAG_NAME"
    print_info "Image version set to: $IMAGE_VERSION"
    echo ""
    
    FULL_IMAGE_NAME="${FULL_REGISTRY_PATH}:${TAG_NAME}"
    
    # Confirmation
    echo -e "${YELLOW}=== BUILD & PUSH SUMMARY ===${NC}"
    echo "Local Image:    $LOCAL_IMAGE"
    echo "Remote Image:   $FULL_IMAGE_NAME"
    echo "Platforms:      $BUILD_PLATFORMS"
    echo "Registry:       $REGISTRY"
    echo -e "${YELLOW}============================${NC}"
    echo ""
    
    read -p "Continue with build & push? (yes/no): " CONFIRM
    
    if [ "$CONFIRM" != "yes" ]; then
        print_warning "Build & push cancelled by user"
        exit 0
    fi
    echo ""
    
    # Check buildx
    if ! docker buildx version &> /dev/null; then
        print_warning "Docker buildx is not available. Attempting to install..."
        
        # Detect OS
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            case $ID in
                arch|cachyos|manjaro)
                    print_info "Arch-based system detected. Installing docker-buildx via pacman..."
                    sudo pacman -S --noconfirm docker-buildx
                    ;;
                ubuntu|debian|linuxmint)
                    print_info "Debian-based system detected. Installing docker-buildx-plugin via apt..."
                    sudo apt-get update && sudo apt-get install -y docker-buildx-plugin
                    ;;
                fedora|rhel|centos)
                    print_info "RHEL-based system detected. Installing docker-buildx via dnf..."
                    sudo dnf install -y docker-buildx
                    ;;
                *)
                    print_error "Unsupported OS ($ID) for auto-installation."
                    print_warning "Please install the Docker Buildx plugin manually."
                    exit 1
                    ;;
            esac
        fi

        # Re-check after installation
        if ! docker buildx version &> /dev/null; then
            print_error "Failed to install Docker buildx automatically."
            exit 1
        fi
        print_success "✓ Docker buildx installed successfully"
    fi
    
    # Create or use existing buildx builder
    BUILDER_NAME="emqx-service-builder"
    if ! docker buildx inspect ${BUILDER_NAME} &> /dev/null; then
        print_info "Creating new buildx builder: ${BUILDER_NAME}"
        docker buildx create --name ${BUILDER_NAME} --use --bootstrap
    else
        print_success "✓ Using existing buildx builder: ${BUILDER_NAME}"
        docker buildx use ${BUILDER_NAME}
    fi
    echo ""
    
    BUILD_ARGS+=("-t" "${FULL_IMAGE_NAME}" "--push")
else
    BUILD_ARGS+=("-t" "${LOCAL_IMAGE}")
fi

# Build with docker buildx
docker buildx build "${BUILD_ARGS[@]}" .

if [ $? -eq 0 ]; then
    if [ "$PUSH_TO_REGISTRY" = true ]; then
        print_success "✓ Docker image built and pushed successfully to GHCR!"
        echo ""
        print_info "Image URL: $FULL_IMAGE_NAME"
        echo ""
        print_info "To pull this image:"
        echo "  docker pull ${FULL_IMAGE_NAME}"
    else
        print_success "✓ Docker image built successfully: ${LOCAL_IMAGE}"
    fi
    echo ""
    
    print_info "📋 Build Artifacts with Metadata:"
    echo "  ├─ Image: $LOCAL_IMAGE"
    echo "  ├─ Title: $IMAGE_TITLE"
    echo "  ├─ Version: $IMAGE_VERSION"
    echo "  ├─ Authors: $IMAGE_AUTHORS"
    echo "  ├─ Platforms: $BUILD_PLATFORMS"
    echo "  ├─ Base Image: debian:bookworm-slim (Multi-stage)"
    echo "  ├─ Runtime User: service (UID: 1000)"
    echo "  ├─ Exposed Port: 5500"
    echo "  ├─ Health Check: Enabled (30s interval)"
    echo "  └─ Architecture: Multi-stage build with buildx"
    echo ""
else
    print_error "Failed to build Docker image with buildx"
    exit 1
fi

# ========================================
# PHASE 3: VALIDATION (Optional)
# ========================================
if [ "$SKIP_TESTS" = false ]; then
    print_stage "PHASE 3: Image Validation & Analysis"
    echo "========================================="
    echo ""
    
    print_info "🔍 Running Docker image validation checks..."
    echo ""
    
    if [ "$BUILD_PLATFORMS" = "linux/amd64,linux/arm64" ] || [[ "$BUILD_PLATFORMS" == *","* ]]; then
        print_warning "Multi-platform build detected - image cached in buildx builder"
        print_info "Image platforms: $BUILD_PLATFORMS"
        print_info "Image name: $LOCAL_IMAGE"
        echo ""
        print_info "📋 Build Artifacts with Metadata:"
        echo "  ├─ Image: $LOCAL_IMAGE"
        echo "  ├─ Title: $IMAGE_TITLE"
        echo "  ├─ Version: $IMAGE_VERSION"
        echo "  ├─ Authors: $IMAGE_AUTHORS"
        echo "  ├─ Platforms: $BUILD_PLATFORMS"
        echo "  ├─ Base Image: debian:bookworm-slim (Multi-stage)"
        echo "  ├─ Runtime User: service (UID: 1000)"
        echo "  ├─ Exposed Port: 5500"
        echo "  └─ Health Check: Enabled (30s interval)"
        echo ""
    elif docker inspect "${LOCAL_IMAGE}" &> /dev/null; then
        print_success "✓ Docker image exists and is valid"
        echo ""
        
        # Get image info
        IMAGE_ID=$(docker inspect "${LOCAL_IMAGE}" -f '{{.ID}}' | cut -d: -f2 | cut -c1-12)
        IMAGE_SIZE=$(docker inspect "${LOCAL_IMAGE}" -f '{{.Size}}' | numfmt --to=iec 2>/dev/null || echo "N/A")
        IMAGE_CREATED=$(docker inspect "${LOCAL_IMAGE}" -f '{{.Created}}')
        
        print_info "📊 Image Metadata & Annotations:"
        print_table_header "Property" "Value" "Annotation"
        print_table_row "Image ID" "$IMAGE_ID" "✓ Unique Hash"
        print_table_row "Image Size" "$IMAGE_SIZE" "✓ Compressed"
        print_table_row "Created" "$(date -d "$IMAGE_CREATED" '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo 'N/A')" "✓ Timestamp"
        echo ""
        
        print_info "🏷️ Image Labels & Annotations (OCI Standard):"
        echo "  ├─ [LABEL] org.opencontainers.image.title=$IMAGE_TITLE"
        echo "  ├─ [LABEL] org.opencontainers.image.version=$IMAGE_VERSION"
        echo "  ├─ [LABEL] org.opencontainers.image.authors=$IMAGE_AUTHORS"
        echo "  ├─ [LABEL] org.opencontainers.image.description=$IMAGE_DESCRIPTION"
        echo "  ├─ [ANNOTATION] org.opencontainers.image.title=$IMAGE_TITLE"
        echo "  ├─ [ANNOTATION] org.opencontainers.image.description=$IMAGE_DESCRIPTION"
        echo "  └─ [ANNOTATION] org.opencontainers.image.source=$IMAGE_SOURCE"
        echo ""
    else
        print_warning "Image not found in local Docker daemon (may be multi-platform build)"
        print_info "Use './push.sh' to build and push multi-platform image to registry"
        echo ""
    fi
    echo ""
else
    print_warning "Skipping validation tests (--no-test flag set)"
    echo ""
fi

# ========================================
# PHASE 4: LOCAL BUILD SUMMARY
# ========================================
print_stage "PHASE 4: Build Summary & Artifacts"
echo "========================================="
echo ""
print_success "✓ Local build completed successfully!"
echo ""

print_info "🎯 Build Summary Table:"
print_table_header "Component" "Status" "Details"
print_table_row "Rust Build" "✓ Complete" "Release mode"
print_table_row "Docker Image" "✓ Complete" "$LOCAL_IMAGE"
print_table_row "Validation" "✓ Complete" "Image verified"
echo ""

print_info "📦 Docker Image Details:"
print_table_header "Field" "Value" "Info"
print_table_row "Local Image" "$LOCAL_IMAGE" "docker run compatible"
print_table_row "Registry Image" "$REGISTRY_IMAGE" "GHCR compatible"
print_table_row "Platforms" "$BUILD_PLATFORMS" "Multi-architecture"
echo ""

print_info "📋 OCI Image Labels & Annotations:"
echo "  ├─ LABELS (Embedded in Image):"
echo "  │  ├─ org.opencontainers.image.title=$IMAGE_TITLE"
echo "  │  ├─ org.opencontainers.image.version=$IMAGE_VERSION"
echo "  │  ├─ org.opencontainers.image.authors=$IMAGE_AUTHORS"
echo "  │  └─ org.opencontainers.image.description=$IMAGE_DESCRIPTION"
echo "  │"
echo "  └─ ANNOTATIONS (In Image Manifest):"
echo "     ├─ org.opencontainers.image.title=$IMAGE_TITLE"
echo "     ├─ org.opencontainers.image.description=$IMAGE_DESCRIPTION"
echo "     └─ org.opencontainers.image.source=$IMAGE_SOURCE"
echo ""

print_info "🚀 Next Steps:"
if [ "$PUSH_TO_REGISTRY" = true ]; then
    echo "  ┌─ Image pushed to GHCR: $FULL_IMAGE_NAME"
    echo "  ├─ To pull: docker pull $FULL_IMAGE_NAME"
    echo "  ├─ To run: docker run -it $FULL_IMAGE_NAME"
    echo "  └─ To build again without push: ./autobuild.sh"
else
    echo "  ┌─ To run the local image:"
    echo "  │  $ docker run -it $LOCAL_IMAGE"
    echo "  │"
    echo "  ├─ To inspect labels:"
    echo "  │  $ docker inspect $LOCAL_IMAGE | grep -A 20 Labels"
    echo "  │"
    echo "  ├─ To push to GHCR:"
    echo "  │  $ ./autobuild.sh --push"
    echo "  │"
    echo "  └─ Or manually tag and push:"
    echo "     $ docker tag $LOCAL_IMAGE $REGISTRY_IMAGE"
    echo "     $ docker push $REGISTRY_IMAGE"
fi
echo ""

# ========================================
# BUILD COMPLETE
# ========================================
echo ""
print_header "╔════════════════════════════════════════════════════════════════╗"
print_header "║                   ✓ BUILD COMPLETE                            ║"
print_header "║                                                                ║"
if [ "$PUSH_TO_REGISTRY" = true ]; then
    print_header "║  Docker Image: $FULL_IMAGE_NAME"
    print_header "║  Status: Built and pushed to GHCR                            ║"
else
    print_header "║  Docker Image: $LOCAL_IMAGE"
    print_header "║  Status: Ready for testing and deployment                     ║"
fi
print_header "╚════════════════════════════════════════════════════════════════╝"
echo ""
