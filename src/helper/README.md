
# Rust Browser Helper Crate

This crate provides utilities for managing and interacting with various web browsers. It is designed to help with browser-specific tasks and automation in Rust applications.

## Prerequisites

To use this crate effectively, you need to have one or more of the following browsers installed locally:

- **Google Chrome**:
  - `google-chrome-stable`
  - `google-chrome-beta`
  - `google-chrome-dev`
  - `google-chrome-unstable`

- **Chromium**:
  - `chromium`
  - `chromium-browser`

- **Microsoft Edge**:
  - `microsoft-edge-stable`
  - `microsoft-edge-beta`
  - `microsoft-edge-dev`

- **Generic Options**:
  - `chrome`
  - `chrome-browser`
  - `msedge`
  - `microsoft-edge`


**Install Any of your preferred chrome engine or Binaries**: You can usually install these using your system’s package manager. For example:

    - **Debian/Ubuntu**:

      ```bash
      sudo apt-get install google-chrome-stable chromium-browser
      sudo apt-get install microsoft-edge-stable
      ```

    - **Fedora**:

      ```bash
      sudo dnf install google-chrome-stable chromium
      sudo dnf install microsoft-edge-stable
      ```

    - **macOS**: You can use Homebrew:

      ```bash
      brew install --cask google-chrome chromium
      brew install --cask microsoft-edge
      ```

    - **Windows**: Download and install the browsers from their official websites.
