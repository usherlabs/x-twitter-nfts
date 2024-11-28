# Rust Bittle AI plugin

## Prerequisites

- Node >=18
- Install `make-agent` - `pnpm i`
  - [Reference Docs](https://docs.bitte.ai/agents/quick-start)

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


## To run

- create environment files
  Copy `.env.example` to `.env.local` an and fill in the necessary values:
    ```
    cp .env.example .env.local
    nano .env.local
    echo "BITTE_CONFIG=''" > .env
    ```

### Environment Variables

| Variable | Description |
|----------|-------------|
| THIRDWEB_CLIENT_ID | Client ID for Thirdweb integration |
| TWEET_BEARER | Bearer token for Twitter API access |
| ACCOUNT_ID | Account ID for agent registration purposes |
| NEAR_CONTRACT_ADDRESS | NFT Contract address for Near blockchain |


### Option 1

```
  ./script/run.sh
```

### Option 2

Start server

```
cargo run
```
and
run agent mainnet

```
npx make-agent dev -p 8007
```

run agent testnet
```
npx make-agent dev -p 8007 -t
```
