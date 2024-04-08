# X404 Meets Solana
## Architecture Overview
The owner can initiate a `X404_Hub` for generating X404 Instance for different NFT collection. For each collection, the owner can create a `X404_State` and a `Owner_Store` for maintaing the data. The owner need to send sufficient lamport to the state and store for future accounts or space changes.

An user can deposit his NFT to corresponding `X404_State` and get `state.fungible_supply` fungible tokens. In addition, an `X404_state` managed NFT collection will assign a new mint account to the depositer. This new mint is recorded in `Owner_Store`. The fungible token is a token2022 token that hook to `X404` program, such that any transfer of the fungible token will trigger the program to check if `Owner_Store` should redistribute the NFT accordingly. To spend the token like NFT, the user need to `Bind` his assigned NFT, which will cost his `state.fungible_supply` fungible tokens and mint the corresponding NFT to the user. If the user want to spend the NFT like fungible token, he can unbind the NFT to put the mint back to `Owner_Store` and get back his fungible token. Eventually, a user can spend `state.fungible_supply` tokens to redeem an deposited NFT as long as it has passed the redeem deadline and the user should pay the `state.redeem_fee` to the original depositer. The original depositer can redeem back his token within redeem deadline without `state.redeem_fee` 

## Instructions

### initialize
Create a new hub (unique) and set manager as well as `emergency_close` bit (not used for now, should add a new instrutctions depends on the governance rule)

### create_x404

Create a new x404 state, should only be called by the manager. the state store the parameters and has a seed with a pubkey `source`, should be used for validating NFT allowed to deposit (not implemented yet). A `Owner_Store` is created for storing unbinded `X404_state` issued NFT.
In addition, this instruction create the mint account for fungible mint with hook call back the rebalance instruction of this program.

### mint_collection

The following command after create_x404. Separated due to stack limitation. Mint the collection NFT for this `X404_state` issued NFT.

### deposit

A user can call this instruction to deposit an authorized NFT and get `state.fungible_supply` fungible token and one `x404_state` issued NFT. The NFT Mint is always a PDA from this program, `x404_state`, and a number represent how many nft minted before. The Mint account will not mint for the depositer directly but store into the `Store_Owner` Account.

### bind

A user can use this instruction to mint the NFT assigned to them in `Owner_store` by paying `state.fungible_supply` fungible token. After binding, the Mint is removed from the `Owner_Store` and the user can transfer like a normal NFT.

### unbind
A user can unbind his issued NFT which will receive `state.fungible_supply` fungible token, burn the NFT token and the mint will be reassigned to the user in `Owner_store`.

### rebalance
The hook call back function, only call by hook program. All transfer of fungible token will trigger this function and cause the program to re-distribute the NFT mint in `Owner_Store` according to the transfer. If the NFT is net decreased, the additional NFT mint will be stored under name of `X404_State`.

### Redeem
By calling redeem, the user will burn his fungible token to redeem a depsoited NFT passed redeem dead line. The user may need to pay the redeem fee to the original owner. The burned fungible token's corresponding NFT will be stored back to `Owner_Store` under then name of `X404_State`. The `X404_State` use `NFT_in_use` to record the total supply of NFT in use and `NFT_Supply` to record the created NFT. `NFT_in_use` is alwasy no larger than `NFT_Supply`. If `NFT_in_use` is smaller than `NFT_Supply`, the new deposit will not create new mint but direct give the old mint in `owner_store`.


## TO DO
1. Intergrate Metaplex
2. Accomplish deposit check and other skipped checks.
3. Client (Ref: `tests/intergration`)
4. security test


## Building

This will build the program and output a `.so` file in a non-comitted `target/deploy` directory which is used by the `config/shank.cjs` configuration file to start a new local validator with the latest changes on the program.

```sh
cargo build-bpf
```

## Testing

`tests/intergration.rs` provide a full flow intergration test for all function.

You may run the following command to build the program and run it.

```sh
cargo build-bpf
cd tests
cargo test-sbf
```

