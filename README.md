# stake-pool program

Full documentation is available at https://spl.solana.com/stake-pool

The command-line interface tool is available in the `./cli` directory.

Javascript bindings are available in the `./js` directory.

Python bindings are available in the `./py` directory.

## Audit

The repository [README](https://github.com/solana-labs/solana-program-library#audits)
contains information about program audits.


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
