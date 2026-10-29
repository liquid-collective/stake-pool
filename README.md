# stake-pool program

Full documentation is available at https://spl.solana.com/stake-pool

The command-line interface tool is available in the `./cli` directory.

Javascript bindings are available in the `./js` directory.

Python bindings are available in the `./py` directory.

## Audit

The repository [README](https://github.com/solana-labs/solana-program-library#audits)
contains information about program audits.

## Notice regarding this fork

The [Liquid Collective Stake Pool](https://github.com/liquid-collective/liquid-collective-solana) uses a fork of the SPL Stake Pool program to permission the stake-pool.

The stake-pool is licensed under the Apache License, Version 2.0. You may obtain a copy of that license at http://www.apache.org/licenses/LICENSE-2.0

Changes to stake-pool are subject to the license described in [PATCH_LICENSE.txt](./PATCH_LICENSE.txt).

### Changes relative to upstream

This fork is upstream [`program@v2.2.0`](https://github.com/solana-program/stake-pool/releases/tag/program%40v2.2.0) plus the following changes in `program/src`:

- `WithdrawStake` and `WithdrawStakeWithSlippage` take an optional 14th account, the stake pool `sol_withdraw_authority`, as a signer. When the pool has a `sol_withdraw_authority` set, the instruction fails unless that account is present and has signed. The Rust builders `withdraw_stake` and `withdraw_stake_with_slippage` take it as their 3rd parameter. This is a breaking change from upstream's signature. Together with the upstream authorities for `DepositSol`, `DepositStake` and `WithdrawSol`, every instruction that moves user funds into or out of the pool can be restricted to a single authority. `program/tests/permissioning.rs` checks that every instruction variant is classified, so a new upstream instruction cannot be added unnoticed.
- Liquid Collective program ids, `security.txt`, license and README.

The earlier Liquid Collective patch that allowed a freeze authority on the pool mint is dropped: since v2.2.0 upstream allows one natively.

Tags are named `program@v<upstream version>-lc.<n>`, so the tag shows which upstream release the fork is based on.

The JavaScript and Python clients only differ from upstream in their default program ids (the Liquid Collective ids from `program/src/lib.rs`, and the test validators load the program under that id). They do not send the `sol_withdraw_authority` account, so they cannot withdraw stake from a pool that has one set.

## Deployment

Inside the project root directory, first build the program with the Solana CLI version pinned in `Cargo.toml` (`workspace.metadata.cli.solana`):

```bash
cargo build-sbf --manifest-path program/Cargo.toml
```

Then we can run the `solana program deploy` command:

```bash
solana program deploy --with-compute-unit-price 1000000  --max-sign-attempts 1000 --url https://api.devnet.solana.com --keypair ./wallets/devnet/deployer.json --upgrade-authority ./wallets/devnet/deployer.json --program-id ./wallets/devnet/stake-pool.json target/deploy/spl_stake_pool.so
```

## Verified Build

**Liquid Collective Release v1.0.0**
The initial version of the program was deployed and verified via [OtterVerify transaction](https://explorer.solana.com/tx/5coLdNVhQc3VBvxNr1W1pmfXXpBajnZ3xByPRLdRW1YZpqj5pB1X5BXA8D4eaZY2dVafWHL5fY3BuzVBi7rc3h6Z?cluster=mainnet). The program deployed matches the one in the [mainnet release PR](https://github.com/liquid-collective/stake-pool/pull/7).

**Liquid Collective Release v2.0.0**
The upgraded program build has been verified via another [OtterVerify transaction](https://explorer.solana.com/tx/AuYmMgxZa4zEpYKJt5REcksTEj5P1DzD5VGmwGVimnweqV1UaypNvSakKESEurMkVKMMNQizQMBXhJqH2FSx5BA). The program deployed matches the state of the program at commit `4d40b6e1de62310ead62fec4596f2ed7f506a3ee` and [LC release v2.0.0](https://github.com/liquid-collective/stake-pool/releases/tag/program%402.0.0-lc). That release is upstream `program@v2.0.4` plus the Liquid Collective patch, and runs on all current deployments.
