#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Syncudio Installation Script for Arch Linux${NC}"
echo "This script will install Syncudio and its dependencies"

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
    echo -e "${RED}Please do not run as root${NC}"
    exit 1
fi

# Check if required dependencies are installed
echo -e "\n${YELLOW}Checking dependencies...${NC}"
if ! pacman -Qi base-devel &> /dev/null; then
    echo -e "${YELLOW}Installing base-devel...${NC}"
    sudo pacman -S --needed base-devel
fi

if ! pacman -Qi fuse2 &> /dev/null; then
    echo -e "${YELLOW}Installing fuse2...${NC}"
    sudo pacman -S --needed fuse2
fi

# Create temporary build directory
BUILD_DIR=$(mktemp -d)
echo -e "\n${YELLOW}Created temporary build directory: $BUILD_DIR${NC}"

# Copy PKGBUILD and AppImage
echo -e "${YELLOW}Copying installation files...${NC}"
cp PKGBUILD "$BUILD_DIR/"
cp src-tauri/target/release/bundle/appimage/Syncudio_0.20.5_amd64.AppImage "$BUILD_DIR/syncudio-0.20.5.AppImage"

# Build and install package
echo -e "\n${YELLOW}Building and installing package...${NC}"
cd "$BUILD_DIR"
makepkg -si

if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}Installation completed successfully!${NC}"
    echo "You can now launch Syncudio from your application menu or by running 'syncudio' in terminal"
else
    echo -e "\n${RED}Installation failed${NC}"
    exit 1
fi

# Cleanup
echo -e "\n${YELLOW}Cleaning up...${NC}"
cd - > /dev/null
rm -rf "$BUILD_DIR"

echo -e "\n${GREEN}All done! Enjoy Syncudio!${NC}" 