# Rust Bittle AI plugin

## Prerequisites

- Node >=18
- [helper Crate Prerequisites ](../helper/README.md)
- Install `make-agent` - `pnpm i`
  - [Reference Docs](https://docs.bitte.ai/agents/quick-start)

## To run

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
run agent

```
npx make-agent dev -p 8007
```
