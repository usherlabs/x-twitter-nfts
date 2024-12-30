# Building Fails

```shell
error: failed to run custom build command for `near-workspaces v0.9.0`

Caused by:
  process didn't exit successfully: `/Users/ryansoury/dev/x-twitter-nfts/src/contracts/verifier/integration-tests/target/debug/build/near-workspaces-a26639bab4fd19ca/build-script-build` (exit status: 101)
  --- stdout
  cargo:rerun-if-env-changed=NEAR_SANDBOX_BIN_PATH

  --- stderr
  thread 'main' panicked at /Users/ryansoury/.cargo/registry/src/index.crates.io-6f17d22bba15001f/near-workspaces-0.9.0/build.rs:10:14:
  Could not install sandbox: unable to download near-sandbox

  Caused by:
      failed to download from https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore/Darwin-arm64/master/eb2bbe1c3f51912c04462ce988aa496fab03d60e/near-sandbox.tar.gz
  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

It seems like the `near-workspaces` crate is trying to download the `near-sandbox` binary, which is no longer available.