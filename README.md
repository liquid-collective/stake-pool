# stake-pool program

Full documentation is available at https://spl.solana.com/stake-pool

The command-line interface tool is available in the `./cli` directory.

Javascript bindings are available in the `./js` directory.

Python bindings are available in the `./py` directory.

## Audit

The repository [README](https://github.com/solana-labs/solana-program-library#audits)
contains information about program audits.

## Notice regarding this fork 

The [Liquid Collective Stake Pool](https://github.com/liquid-collective/liquid-collective-solana) uses a fork of the SPL Stake Pool program to permission the stake-pool and allow freezable tokens.

The stake-pool is licensed under the Apache License, Version 2.0. You may obtain a copy of that license at http://www.apache.org/licenses/LICENSE-2.0

Changes to stake-pool are subject to the license described in [PATCH_LICENSE.txt](./PATCH_LICENSE.txt).

## Deployment
Inside the `program` directory:

First we need to build the program:

```bash
cargo build-sbf
```

Then we can run the `solana program deploy` command:

```bash
solana program deploy --with-compute-unit-price 1000000  --max-sign-attempts 1000 --url https://api.devnet.solana.com --keypair ./wallets/devnet/deployer.json --upgrade-authority ./wallets/devnet/deployer.json --program-id ./wallets/devnet/stake-pool.json target/deploy/spl_stake_pool.so
```

## Verified Build

**Liquid Collective Release v1.0.0**
The initial version of the program was deployed and verified via [OtterVerify transaction](https://explorer.solana.com/tx/5coLdNVhQc3VBvxNr1W1pmfXXpBajnZ3xByPRLdRW1YZpqj5pB1X5BXA8D4eaZY2dVafWHL5fY3BuzVBi7rc3h6Z?cluster=mainnet). The program deployed matches the one in the [mainnet release PR](https://github.com/liquid-collective/stake-pool/pull/7).

**Liquid Collective Release v2.0.0**
The upgraded program build has been verified via another [OtterVerify transaction](https://explorer.solana.com/tx/AuYmMgxZa4zEpYKJt5REcksTEj5P1DzD5VGmwGVimnweqV1UaypNvSakKESEurMkVKMMNQizQMBXhJqH2FSx5BA). The program deployed matches the state of the program at commit `4d40b6e1de62310ead62fec4596f2ed7f506a3ee` and [LC release v2.0.0](https://github.com/liquid-collective/stake-pool/releases/tag/program%402.0.0-lc).