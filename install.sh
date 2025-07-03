#!/usr/bin/env sh

# Adapted installer for archived releases matching deploy script
# Inspiration: https://github.com/comtrya/get.comtrya.dev/blob/main/public/index.html

export VERIFY_CHECKSUM=0
export ALIAS_NAME="dotgk"
export OWNER=jrodal98
export REPO=dotgk
export SUCCESS_CMD="$REPO --help"
export BINLOCATION="/usr/local/bin"

if [ -z "${VERSION}" ]; then
  version=$(curl -sI https://github.com/$OWNER/$REPO/releases/latest | grep -i "location:" | awk -F"/" '{ print $NF }' | tr -d '\r')
else
  version=${VERSION}
fi

status_code=$(curl -sI -o /dev/null -w "%{http_code}" https://github.com/$OWNER/$REPO/releases/tag/$version)

if [ "$status_code" -eq 404 ]; then
  echo "Failed to find release $version for $REPO. Please install manually:"
  echo "1. Visit https://github.com/$OWNER/$REPO/releases"
  echo "2. Download the archive for your platform."
  echo "3. Extract and move the binary to $BINLOCATION"
  exit 1
fi

hasCli() {
  if ! command -v curl >/dev/null 2>&1; then
    echo "curl is required to run this script."
    exit 1
  fi
  if ! command -v tar >/dev/null 2>&1 && ! command -v 7z >/dev/null 2>&1; then
    echo "tar or 7z is required to extract archives."
    exit 1
  fi
}

getPackage() {
  uname=$(uname -s)
  arch=$(uname -m)
  suffix=""
  archive_ext=""
  binary_name="$REPO"
  case "$uname" in
  MINGW* | MSYS* | CYGWIN* | Windows_NT)
    suffix="x86_64-pc-windows-msvc"
    archive_ext="zip"
    binary_name="${REPO}.exe"
    BINLOCATION="$HOME/bin"
    mkdir -p "$BINLOCATION"
    ;;
  Darwin)
    case "$arch" in
    arm64) suffix="aarch64-apple-darwin" ;;
    *) suffix="x86_64-apple-darwin" ;;
    esac
    archive_ext="tar.gz"
    ;;
  Linux)
    case "$arch" in
    x86_64) suffix="x86_64-unknown-linux-musl" ;;
    aarch64) suffix="aarch64-unknown-linux-gnu" ;;
    *)
      echo "Unsupported architecture: $arch"
      exit 1
      ;;
    esac
    archive_ext="tar.gz"
    ;;
  *)
    echo "Unsupported OS: $uname"
    exit 1
    ;;
  esac

  archive_name="${REPO}-${version}-${suffix}.${archive_ext}"
  target_dir="/tmp/${REPO}_install"
  target_archive="${target_dir}/${archive_name}"

  if [ "$(id -u)" != "0" ]; then
    BINLOCATION="${HOME}/bin"
    mkdir -p "$BINLOCATION"
  fi

  mkdir -p "$target_dir"
  echo "Downloading $archive_name from GitHub releases..."
  curl -sSL "https://github.com/$OWNER/$REPO/releases/download/$version/$archive_name" -o "$target_archive"
  if [ $? -ne 0 ]; then
    echo "Download failed."
    exit 1
  fi

  if [ "$VERIFY_CHECKSUM" = "1" ]; then
    # Implement checksum verification if you publish .sha256 files
    echo "Checksum verification not implemented in this script."
  fi

  echo "Extracting archive..."
  case "$archive_ext" in
  zip)
    if command -v 7z >/dev/null 2>&1; then
      7z x "$target_archive" -o"$target_dir"
    else
      echo "7z is required to extract zip archives."
      exit 1
    fi
    ;;
  tar.gz)
    tar -xzf "$target_archive" -C "$target_dir"
    ;;
  *)
    echo "Unknown archive format: $archive_ext"
    exit 1
    ;;
  esac

  extracted_dir="${target_dir}/${REPO}-${version}-${suffix}"
  if [ ! -d "$extracted_dir" ]; then
    echo "Expected extracted directory $extracted_dir not found."
    exit 1
  fi

  echo "Installing binary to $BINLOCATION..."
  if [ ! -w "$BINLOCATION" ]; then
    echo "No write permission to $BINLOCATION. You may need to run with sudo."
    echo "Copying binary to current directory instead."
    cp "${extracted_dir}/${binary_name}" .
    chmod +x "./${binary_name}"
    echo "Binary copied to $(pwd)/${binary_name}"
  else
    cp "${extracted_dir}/${binary_name}" "$BINLOCATION/$REPO"
    chmod +x "$BINLOCATION/$REPO"
    echo "Installed $REPO to $BINLOCATION"
  fi

  if [ -n "$ALIAS_NAME" ] && [ "$ALIAS_NAME" != "$REPO" ]; then
    if [ ! -w "$BINLOCATION" ]; then
      echo "Cannot create alias $ALIAS_NAME in $BINLOCATION without write permission."
    else
      ln -sf "$BINLOCATION/$REPO" "$BINLOCATION/$ALIAS_NAME"
      echo "Alias $ALIAS_NAME created for $REPO"
    fi
  fi

  echo "Cleaning up..."
  rm -rf "$target_dir"

  echo "Installation complete."
  $SUCCESS_CMD
}

hasCli
getPackage
