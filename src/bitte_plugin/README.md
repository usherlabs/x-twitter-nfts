# Rust Bittle AI Plugin

## Prerequisites

- **Node.js**: Version 18 or higher
- **Make-Agent Tool**: Install using `pnpm i`
  - [Reference Documentation](https://docs.bitte.ai/agents/quick-start)

### Supported Browsers

To use this crate effectively, ensure you have one or more of the following browsers installed:

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

**Installation**: Use your system’s package manager to install the preferred browser binaries. For example:

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

- **macOS**: Use Homebrew:

  ```bash
  brew install --cask google-chrome chromium
  brew install --cask microsoft-edge
  ```

- **Windows**: Download and install the browsers from their official websites.

## Running the Plugin

Bitte's `make-agent` tool generates a `.env` file containing critical information for Bitte AI to register and understand the plugin. Since the registration occurs immediately after file generation, we use a separate dotenv file, **`plugin.env`**, for custom environment variables.

### Setting Up Environment Files

1. Copy `.env.sample` to `plugin.env` and fill in the necessary values:

   ```bash
   cp .env.sample plugin.env
   vim plugin.env
   ```

### Environment Variables

| Variable              | Description                                      |
|-----------------------|--------------------------------------------------|
| `THIRDWEB_CLIENT_ID`  | Client ID for Thirdweb integration               |
| `TWEET_BEARER`        | Bearer token for Twitter API access              |
| `ACCOUNT_ID`          | Account ID for agent registration purposes       |
| `NEAR_CONTRACT_ADDRESS` | NFT Contract address for Near blockchain       |
| `HOST_URL`            | Optional - only required for production          |

## Execution Options

### Option 1: Using Script

```bash
./script/run.sh
```

### Option 2: Manual Execution

1. Start the server:

   ```bash
   cargo run
   ```

2. Run the agent on the mainnet:

   ```bash
   npx make-agent dev -p 8007
   ```

3. Run the agent on the testnet:

   ```bash
   npx make-agent dev -p 8007 -t
   ```
