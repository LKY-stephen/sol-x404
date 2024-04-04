use crate::{
    instruction, DepositParams, InitCollectionParams, InitTokenParams, RebalanceParams,
    RedeemParams, UnbindParams, ID,
};
use anchor_lang::{prelude::*, system_program, InstructionData};
use anchor_spl::{
    associated_token, token,
    token_2022::{self},
};
use solana_program::instruction::Instruction;

pub fn initialize(state: Pubkey, signer: Pubkey) -> Instruction {
    let data = instruction::Initialize {};
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new(state, false),
            AccountMeta::new(signer, true),
            AccountMeta::new(system_program::ID, false),
        ],
    )
}

pub fn create_x404(
    redeem_max_deadline: u64,
    redeem_fee: u64,
    decimals: u8,
    hub: Pubkey,
    source: Pubkey,
    state: Pubkey,
    owner_store: Pubkey,
    collection_mint: Pubkey,
    fungible_mint: Pubkey,
    signer: Pubkey,
    hook_extra_account: Pubkey,
    hook_program: Pubkey,
    fungible_supply: u64,
) -> Instruction {
    let data = instruction::CreateX404 {
        params: InitTokenParams {
            redeem_max_deadline,
            redeem_fee,
            decimals,
            fungible_supply,
            hook_extra_account,
            hook_program,
        },
    };
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new(hub, false),
            AccountMeta::new(source, false),
            AccountMeta::new(state, false),
            AccountMeta::new(owner_store, false),
            AccountMeta::new(collection_mint, false),
            AccountMeta::new(fungible_mint, false),
            AccountMeta::new(signer, true),
            // token
            AccountMeta::new_readonly(anchor_spl::token_2022::ID, false),
            // system
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    )
}

pub fn mint_collection(
    name: String,
    symbol: String,
    uri: String,
    source: Pubkey,
    state: Pubkey,
    nft_mint: Pubkey,
    nft_token: Pubkey,
    signer: Pubkey,
) -> Instruction {
    let data = instruction::MintCollection {
        _params: InitCollectionParams {
            name,
            symbol,
            uri,
            source,
        },
    };
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new(state, false),
            AccountMeta::new(nft_mint, false),
            AccountMeta::new(nft_token, false),
            AccountMeta::new(signer, true),
            // token
            AccountMeta::new_readonly(token_2022::ID, false),
            // ata
            AccountMeta::new_readonly(associated_token::ID, false),
            // system
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    )
}

pub fn deposit_spl_nft(
    redeem_deadline: u64,
    source: Pubkey,
    state: Pubkey,
    owner_store: Pubkey,
    deposit_mint: Pubkey,
    deposit_holder: Pubkey,
    deposit_receiver: Pubkey,
    nft_bank: Pubkey,
    nft_mint: Pubkey,
    nft_token: Pubkey,
    fungible_mint: Pubkey,
    fungible_token: Pubkey,
    signer: Pubkey,
) -> Instruction {
    let data = instruction::Deposit {
        params: DepositParams {
            redeem_deadline,
            source,
        },
    };
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new(state, false),
            AccountMeta::new(owner_store, false),
            AccountMeta::new_readonly(deposit_mint, false),
            AccountMeta::new(deposit_holder, false),
            AccountMeta::new(deposit_receiver, false),
            AccountMeta::new(nft_bank, false),
            AccountMeta::new(nft_mint, false),
            AccountMeta::new(nft_token, false),
            AccountMeta::new(fungible_mint, false),
            AccountMeta::new(fungible_token, false),
            AccountMeta::new(signer, true),
            // token
            AccountMeta::new_readonly(token::ID, false),
            // token
            AccountMeta::new_readonly(token_2022::ID, false),
            // ata
            AccountMeta::new_readonly(associated_token::ID, false),
            // system
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    )
}

pub fn redeem_spl_nft(
    source: Pubkey,
    state: Pubkey,
    owner_store: Pubkey,
    withdraw_mint: Pubkey,
    withdraw_holder: Pubkey,
    withdraw_receiver: Pubkey,
    nft_bank: Pubkey,
    original_owner_account: Pubkey,
    fungible_mint: Pubkey,
    fungible_token: Pubkey,
    signer: Pubkey,
) -> Instruction {
    let data = instruction::Redeem {
        _params: RedeemParams { source },
    };
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new(state, false),
            AccountMeta::new(owner_store, false),
            AccountMeta::new(withdraw_mint, false),
            AccountMeta::new(withdraw_holder, false),
            AccountMeta::new(withdraw_receiver, false),
            AccountMeta::new(nft_bank, false),
            AccountMeta::new(fungible_mint, false),
            AccountMeta::new(fungible_token, false),
            AccountMeta::new(original_owner_account, false),
            AccountMeta::new(signer, true),
            // token
            AccountMeta::new_readonly(token::ID, false),
            // token
            AccountMeta::new_readonly(token_2022::ID, false),
            // ata
            AccountMeta::new_readonly(associated_token::ID, false),
            // system
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    )
}

pub fn bind(
    number: u64,
    source: Pubkey,
    state: Pubkey,
    owner_store: Pubkey,
    bind_mint: Pubkey,
    bind_holder: Pubkey,
    bind_receiver: Pubkey,
    fungible_mint: Pubkey,
    fungible_token: Pubkey,
    signer: Pubkey,
) -> Instruction {
    let data = instruction::BindNft {
        _params: crate::BindParams { source, number },
    };
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new(state, false),
            AccountMeta::new(owner_store, false),
            AccountMeta::new(bind_mint, false),
            AccountMeta::new(bind_holder, false),
            AccountMeta::new(bind_receiver, false),
            AccountMeta::new(fungible_mint, false),
            AccountMeta::new(fungible_token, false),
            AccountMeta::new(signer, true),
            // token
            AccountMeta::new_readonly(token_2022::ID, false),
            // ata
            AccountMeta::new_readonly(associated_token::ID, false),
            // system
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    )
}

pub fn unbind(
    number: u64,
    source: Pubkey,
    state: Pubkey,
    owner_store: Pubkey,
    bind_mint: Pubkey,
    bind_holder: Pubkey,
    bind_receiver: Pubkey,
    fungible_mint: Pubkey,
    fungible_token: Pubkey,
    signer: Pubkey,
) -> Instruction {
    let data = instruction::UnbindNft {
        _params: UnbindParams { source, number },
    };
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new(state, false),
            AccountMeta::new(owner_store, false),
            AccountMeta::new(bind_mint, false),
            AccountMeta::new(bind_holder, false),
            AccountMeta::new(bind_receiver, false),
            AccountMeta::new(fungible_mint, false),
            AccountMeta::new(fungible_token, false),
            AccountMeta::new(signer, true),
            // token
            AccountMeta::new_readonly(token_2022::ID, false),
            // ata
            AccountMeta::new_readonly(associated_token::ID, false),
            // system
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    )
}

pub fn rebalance(
    state: Pubkey,
    owner_store: Pubkey,
    sender: Pubkey,
    receiver: Pubkey,
    amount: u64,
    fungible_mint: Pubkey,
    sender_token: Pubkey,
    receiver_token: Pubkey,
    hook: Pubkey,
) -> Instruction {
    let data = instruction::Rebalance {
        params: RebalanceParams {
            sender,
            receiver,
            amount,
        },
    };
    Instruction::new_with_bytes(
        ID,
        &data.data(),
        vec![
            AccountMeta::new(state, false),
            AccountMeta::new(owner_store, false),
            AccountMeta::new(fungible_mint, false),
            AccountMeta::new(sender_token, false),
            AccountMeta::new(receiver_token, false),
            AccountMeta::new(hook, true),
            // ata
            AccountMeta::new_readonly(associated_token::ID, false),
        ],
    )
}
