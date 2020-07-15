
# Cumulus + evm

## Relay setup

- `git clone https://github.com/paritytech/polkadot`
- `git checkout f2d81a8a2097c69ab721edeed00dda34682b1d3c`
- `cargo build --release`
- `cd target/release`
- `./polkadot build-spec --chain westend-local > plain-spec.json`
- Remove `forkBlocks` and `badBlocks` properties ([https://github.com/paritytech/cumulus/issues/126](https://github.com/paritytech/cumulus/issues/126))
- `./polkadot build-spec --chain plain-spec.json --raw --disable-default-bootnode > spec.json`

**Alice**

    ./polkadot \
      --chain spec.json \
      --base-path ../../tmp/alice \
      --ws-port 9944 \
      --port 30333 \
      --alice
Save value shown in the terminal under `Local node identity is`. Should be something like `12D3KooWDLwx3wRvYCMadzYkfwP7N3m8qYKYNwFsftfgZavE9ho6`.

**Bob**

    ./polkadot \
      --chain spec.json \
      --base-path ../../tmp/bob \
      --ws-port 9945 \
      --port 30334 \
      --bob
      
Save value shown in the terminal under `Local node identity is`.

Check that both nodes peer and that the block are being produced and finalized.

## Parachain setup

- `git clone https://github.com/PureStake/cumulus --branch tgmichel-evm-frontier`
- `cargo build --release`
- Copy the `spec.json` file generated for the relay to `target/release`
- `cd target/release`

**Collator 100**

Below, replace `{ALICE_IDENTITY_ID}` and `{BOB_IDENTITY_ID}` with the Local node identities.

    ./cumulus-test-parachain-collator \
      --base-path ../../tmp/collator-100 \
      --ws-port 9988 \
      --port 30337 \
      --rpc-port 9999 \
      --parachain-id 100 \
      -- \
      --chain spec.json \
      --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/{ALICE_IDENTITY_ID} \
      --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/{BOB_IDENTITY_ID}

Save value shown in the terminal under `Parachain genesis state`. Should be something like `0x00000...`.

Check that the collator node follows the relay chain head - some `Relaychain` blue log should appear in the terminal showing the relay block number as well as `Parachain` yellow log that will stay at block #0 until we register the parachain in the next step.

## Registering the parachain

Go to [https://polkadot.js.org/apps](https://polkadot.js.org/apps), access the relay chain node Alice and go to `Sudo` > `register` > `registerPara`.

- id: 100
- scheduling: Always
- code: toggle on the file upload and locate `cumulus/target/release/wbuild/cumulus-test-parachain-runtime/cumulus_test_parachain_runtime.compact.wasm`
- initial_head_data: the value for `Parachain genesis state`.

Issue the sudo command.

At this point if we go back to the parachain terminal, we should see that the parachain starts producing blocks. This means that the relay chain is succesfully collating.
