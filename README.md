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

We verified the build using[OtterVerify transaction](https://explorer.solana.com/tx/5coLdNVhQc3VBvxNr1W1pmfXXpBajnZ3xByPRLdRW1YZpqj5pB1X5BXA8D4eaZY2dVafWHL5fY3BuzVBi7rc3h6Z?cluster=mainnet). The program deployed matches the one in the [mainnet release PR](https://github.com/liquid-collective/stake-pool/pull/7).
