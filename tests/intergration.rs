use std::vec;

use anchor_lang::{AccountDeserialize, Id, Key};
use anchor_spl::{
    token::spl_token,
    token_2022::{
        spl_token_2022::{self, instruction::transfer_checked},
        Token2022,
    },
};
use solana_program::instruction::Instruction;
use solana_program_test::{
    tokio::{self},
    BanksClientError, ProgramTest, ProgramTestContext,
};
use solana_sdk::{
    account::AccountSharedData,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_instruction::{self, create_account},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};
use spl_transfer_hook_interface::get_extra_account_metas_address;

use x404::{
    state::{X404Hub, X404State},
    ID,
};
use x404_hook::{initialize_extra_account, ID as HookID};

const REDEEMFEE: u64 = 100;
const REDEEM_MAX_DEADLINE: u64 = 100;
const DECIMALS: u8 = 2;
const FUNGIBLE_SUPPLY: u64 = 1000;

#[cfg(test)]
#[tokio::test]
async fn functionality_test() {
    use anchor_spl::token_interface::TokenAccount;
    use x404::state::OwnerStore;

    let mut validator = ProgramTest::default();
    validator.add_program("X404", ID, None);
    validator.add_program("X404_HOOK", HookID, None);
    validator.add_program("spl_token_2022", spl_token_2022::ID, None);
    // validator.add_program("metaplex_token_metadata_program", metadata::ID, None);

    let owner = add_account(&mut validator, 100);

    let usera = add_account(&mut validator, 5);
    let userb = add_account(&mut validator, 5);

    let hub_state = add_pda(&[b"hub".as_ref()], ID);
    let source = add_pda(&[b"test_mint".as_ref()], Token2022::id());
    let x404_state = add_pda(&[b"state".as_ref(), source.as_ref()], ID);
    let owner_store = add_pda(&[b"owner_store".as_ref(), x404_state.as_ref()], ID);
    let collection_mint = add_pda(&[b"collection_mint".as_ref(), x404_state.as_ref()], ID);

    let nft_token = get_associated_token_address_with_program_id(
        &collection_mint,
        &collection_mint,
        &Token2022::id(),
    );

    let fungible_mint = add_pda(&[b"fungible_mint".as_ref(), x404_state.as_ref()], ID);

    let extra_account = get_extra_account_metas_address(&fungible_mint, &HookID);

    let mut context = validator.start_with_context().await;

    // initiate the hub
    test_init(
        &mut context,
        &owner,
        hub_state,
        source,
        x404_state,
        owner_store,
        collection_mint,
        nft_token,
        fungible_mint,
        extra_account,
    )
    .await;

    let (nft_a, _deposit_a) = test_deposit(
        &mut context,
        source,
        x404_state,
        owner_store,
        &usera,
        fungible_mint,
        0,
    )
    .await
    .unwrap();
    let (nft_b, _deposit_b) = test_deposit(
        &mut context,
        source,
        x404_state,
        owner_store,
        &userb,
        fungible_mint,
        1,
    )
    .await
    .unwrap();

    let (nft_c, deposit_c) = test_deposit(
        &mut context,
        source,
        x404_state,
        owner_store,
        &userb,
        fungible_mint,
        2,
    )
    .await
    .unwrap();

    print!("check owner store");
    let owner_store_data = read_account::<OwnerStore>(&mut context, owner_store)
        .await
        .unwrap()
        .get_map();

    assert!(owner_store_data
        .get(&usera.pubkey())
        .unwrap()
        .contains(&nft_a));
    assert!(owner_store_data
        .get(&userb.pubkey())
        .unwrap()
        .contains(&nft_b));
    assert!(owner_store_data
        .get(&userb.pubkey())
        .unwrap()
        .contains(&nft_c));

    // transfer token
    println!("transfer fungible token from b to a");
    test_transfer(&mut context, &userb, &usera, fungible_mint, FUNGIBLE_SUPPLY)
        .await
        .unwrap();

    let owner_store_data = read_account::<OwnerStore>(&mut context, owner_store)
        .await
        .unwrap()
        .get_map();

    assert!(owner_store_data
        .get(&usera.pubkey())
        .unwrap()
        .contains(&nft_a));
    assert!(owner_store_data
        .get(&userb.pubkey())
        .unwrap()
        .contains(&nft_b));
    // the third token should be moved to user a's account
    assert!(owner_store_data
        .get(&usera.pubkey())
        .unwrap()
        .contains(&nft_c));

    let state_data = read_account::<X404State>(&mut context, x404_state)
        .await
        .unwrap();

    assert_eq!(state_data.nft_supply, 3);
    assert_eq!(state_data.nft_in_use, 3);

    // bind then transfer
    test_bind(
        &mut context,
        source,
        x404_state,
        owner_store,
        nft_c,
        &usera,
        fungible_mint,
        2,
    )
    .await
    .unwrap();

    test_transfer(
        &mut context,
        &usera,
        &userb,
        fungible_mint,
        FUNGIBLE_SUPPLY / 2,
    )
    .await
    .unwrap();

    let owner_store_data = read_account::<OwnerStore>(&mut context, owner_store)
        .await
        .unwrap()
        .get_map();

    assert!(owner_store_data.get(&usera.pubkey()).unwrap().len() == 0);

    assert!(owner_store_data
        .get(&userb.pubkey())
        .unwrap()
        .contains(&nft_b));
    assert!(owner_store_data.get(&x404_state).unwrap().contains(&nft_a));

    let account_a = get_associated_token_address_with_program_id(
        &usera.pubkey(),
        &fungible_mint,
        &spl_token_2022::ID,
    );

    let a_balance = read_account::<TokenAccount>(&mut context, account_a)
        .await
        .unwrap();

    assert_eq!(a_balance.amount, FUNGIBLE_SUPPLY / 2);

    let account_b = get_associated_token_address_with_program_id(
        &userb.pubkey(),
        &fungible_mint,
        &spl_token_2022::ID,
    );
    let b_balance = read_account::<TokenAccount>(&mut context, account_b)
        .await
        .unwrap();

    assert_eq!(b_balance.amount, FUNGIBLE_SUPPLY + FUNGIBLE_SUPPLY / 2);
    // unbind
    test_unbind(
        &mut context,
        source,
        x404_state,
        owner_store,
        nft_c,
        &usera,
        fungible_mint,
        2,
    )
    .await
    .unwrap();

    let owner_store_data = read_account::<OwnerStore>(&mut context, owner_store)
        .await
        .unwrap()
        .get_map();

    assert!(owner_store_data
        .get(&usera.pubkey())
        .unwrap()
        .contains(&nft_c));

    let account_a = get_associated_token_address_with_program_id(
        &usera.pubkey(),
        &fungible_mint,
        &spl_token_2022::ID,
    );

    let a_balance = read_account::<TokenAccount>(&mut context, account_a)
        .await
        .unwrap();

    assert_eq!(a_balance.amount, FUNGIBLE_SUPPLY + FUNGIBLE_SUPPLY / 2);
    // redeem

    test_redeem(
        &mut context,
        source,
        x404_state,
        owner_store,
        deposit_c,
        &usera,
        fungible_mint,
        userb.pubkey(),
    )
    .await
    .unwrap();

    test_transfer(&mut context, &userb, &usera, fungible_mint, FUNGIBLE_SUPPLY)
        .await
        .unwrap();

    let owner_store_data = read_account::<OwnerStore>(&mut context, owner_store)
        .await
        .unwrap()
        .get_map();

    assert!(owner_store_data.get(&usera.pubkey()).unwrap().len() == 0);
    assert!(owner_store_data
        .get(&userb.pubkey())
        .unwrap()
        .contains(&nft_b));
    // both  and c should be in store
    assert!(owner_store_data.get(&x404_state).unwrap().contains(&nft_c));
    assert!(owner_store_data.get(&x404_state).unwrap().contains(&nft_a));

    let state_data = read_account::<X404State>(&mut context, x404_state)
        .await
        .unwrap();

    assert_eq!(state_data.nft_supply, 3);
    assert_eq!(state_data.nft_in_use, 2);

    let a_balance = read_account::<TokenAccount>(&mut context, account_a)
        .await
        .unwrap();

    assert_eq!(a_balance.amount, FUNGIBLE_SUPPLY / 2 - REDEEMFEE);
    let b_balance = read_account::<TokenAccount>(&mut context, account_b)
        .await
        .unwrap();

    assert_eq!(
        b_balance.amount,
        FUNGIBLE_SUPPLY + FUNGIBLE_SUPPLY / 2 + REDEEMFEE
    );
}

fn add_account(validator: &mut ProgramTest, amount: u64) -> Keypair {
    let keypair = Keypair::new();
    let account =
        AccountSharedData::new(amount * 1_000_000_000, 0, &solana_sdk::system_program::id());
    validator.add_account(keypair.pubkey(), account.into());
    keypair
}

fn add_pda(seeds: &[&[u8]], id: Pubkey) -> Pubkey {
    Pubkey::find_program_address(seeds, &id).0
}

async fn transfer_lamports(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    address: Pubkey,
    amount: u64,
) -> Result<(), BanksClientError> {
    let transfer_instruction = system_instruction::transfer(&payer.pubkey(), &address, amount);

    execute(context, payer, &[transfer_instruction], vec![&payer]).await?;
    Ok(())
}
async fn create_spl_nft(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    authority: Pubkey,
) -> Result<(Pubkey, Pubkey), BanksClientError> {
    let mint = Keypair::new();
    let token =
        get_associated_token_address_with_program_id(&authority, &mint.pubkey(), &spl_token::ID);
    let rent = context.banks_client.get_rent().await.unwrap();
    let mint_space = spl_token::state::Mint::LEN;
    execute(
        context,
        &payer,
        &[
            create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(mint_space),
                mint_space as u64,
                &spl_token::ID,
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::ID,
                &mint.pubkey(),
                &payer.pubkey(),
                None,
                0,
            )
            .unwrap(),
        ],
        vec![payer, &mint],
    )
    .await
    .unwrap();
    execute(
        context,
        &payer,
        &[
            create_associated_token_account(
                &payer.pubkey(),
                &authority,
                &mint.pubkey(),
                &spl_token::ID,
            ),
            spl_token::instruction::mint_to(
                &spl_token::ID,
                &mint.pubkey(),
                &token,
                &payer.pubkey(),
                &[],
                1,
            )
            .unwrap(),
        ],
        vec![payer],
    )
    .await
    .unwrap();

    // skip freeze mint here.

    Ok((mint.pubkey(), token))
}

async fn read_account<T: AccountDeserialize>(
    context: &mut ProgramTestContext,
    address: Pubkey,
) -> Result<T, anchor_lang::prelude::Error> {
    let read = context
        .banks_client
        .get_account(address)
        .await
        .unwrap()
        .unwrap();

    T::try_deserialize(&mut read.data.as_ref())
}
async fn execute(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    instructions: &[Instruction],
    signers: Vec<&Keypair>,
) -> Result<(), BanksClientError> {
    let transaction = Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        &signers,
        context.banks_client.get_latest_blockhash().await?,
    );
    context.banks_client.process_transaction(transaction).await
}

async fn test_init(
    mut context: &mut ProgramTestContext,
    owner: &Keypair,
    hub_state: Pubkey,
    source: Pubkey,
    x404_state: Pubkey,
    owner_store: Pubkey,
    collection_mint: Pubkey,
    nft_token: Pubkey,
    fungible_mint: Pubkey,
    extra_account: Pubkey,
) {
    let init_instruction = x404::instructions::initialize(hub_state, owner.pubkey());

    execute(context, &owner, &[init_instruction], vec![&owner])
        .await
        .unwrap();

    println!("check hub");

    let hub_data = read_account::<X404Hub>(context, hub_state).await.unwrap();

    assert_eq!(hub_data.manager, owner.pubkey());
    assert_eq!(hub_data.emergency_close, false);

    // create x404

    let init_x404_instruction = x404::instructions::create_x404(
        REDEEMFEE,
        REDEEM_MAX_DEADLINE,
        DECIMALS,
        hub_state,
        source,
        x404_state,
        owner_store,
        collection_mint,
        fungible_mint,
        owner.pubkey(),
        extra_account,
        HookID,
        FUNGIBLE_SUPPLY,
    );

    execute(&mut context, &owner, &[init_x404_instruction], vec![&owner])
        .await
        .unwrap();

    println!("check state");
    let state_data = read_account::<X404State>(context, x404_state)
        .await
        .unwrap();

    assert_eq!(state_data.collection_mint, collection_mint.key());
    assert_eq!(state_data.fungible_mint, fungible_mint.key());
    assert_eq!(state_data.owner, owner.pubkey());
    assert_eq!(state_data.redeem_fee, REDEEMFEE);
    assert_eq!(state_data.redeem_max_deadline, REDEEM_MAX_DEADLINE);
    assert_eq!(state_data.decimal, DECIMALS);
    assert_eq!(state_data.fungible_hook, extra_account);
    assert_eq!(state_data.nft_supply, 0);

    // add additional rent to state for future minting
    transfer_lamports(&mut context, owner, x404_state, 5_000_000)
        .await
        .unwrap();
    // initiate extra account

    execute(
        context,
        owner,
        &[initialize_extra_account(
            extra_account,
            fungible_mint,
            owner.pubkey(),
            x404_state,
            owner_store,
        )],
        vec![owner],
    )
    .await
    .unwrap();
    // create collection

    let init_collection_instruction = x404::instructions::mint_collection(
        "test_nft".to_string(),
        "test_nft".to_string(),
        "https://www.google.com/images/branding/googlelogo/1x/googlelogo_light_color_272x92dp.png"
            .to_string(),
        source,
        x404_state,
        collection_mint,
        nft_token,
        owner.pubkey(),
    );

    execute(
        &mut context,
        &owner,
        &[init_collection_instruction],
        vec![&owner],
    )
    .await
    .unwrap();
    println!("initiate accomplished");
}

async fn test_deposit(
    mut context: &mut ProgramTestContext,
    source: Pubkey,
    x404_state: Pubkey,
    owner_store: Pubkey,
    user: &Keypair,
    fungible_mint: Pubkey,
    supply: u64,
) -> Result<(Pubkey, Pubkey), BanksClientError> {
    let (deposit_mint, deposit_holder) = create_spl_nft(&mut context, user, user.pubkey())
        .await
        .unwrap();

    println!("start to deposit {deposit_mint}");
    let deposit_receiver =
        get_associated_token_address_with_program_id(&x404_state, &deposit_mint, &spl_token::ID);
    let nft_bank = add_pda(&[b"nft_bank".as_ref(), deposit_mint.as_ref()], ID);

    let nft_mint = add_pda(
        &[
            b"nft_mint".as_ref(),
            x404_state.as_ref(),
            supply.to_le_bytes().as_ref(),
        ],
        ID,
    );
    let nft_token =
        get_associated_token_address_with_program_id(&x404_state, &nft_mint, &spl_token_2022::ID);

    let fungible_token = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &fungible_mint,
        &spl_token_2022::ID,
    );

    let deposit_instructiona = x404::instructions::deposit_spl_nft(
        1,
        source,
        x404_state,
        owner_store,
        deposit_mint,
        deposit_holder,
        deposit_receiver,
        nft_bank,
        nft_mint,
        nft_token,
        fungible_mint,
        fungible_token,
        user.pubkey(),
    );

    execute(context, user, &[deposit_instructiona], vec![user])
        .await
        .unwrap();

    println!("accomplished deposit {deposit_mint}");
    Ok((nft_mint, deposit_mint))
}

async fn test_transfer(
    context: &mut ProgramTestContext,
    usera: &Keypair,
    userb: &Keypair,
    fungible_mint: Pubkey,
    amount: u64,
) -> Result<(), BanksClientError> {
    println!("start to transfer {amount}");
    let usera_account = get_associated_token_address_with_program_id(
        &usera.pubkey(),
        &fungible_mint,
        &spl_token_2022::ID,
    );
    let userb_account = get_associated_token_address_with_program_id(
        &userb.pubkey(),
        &fungible_mint,
        &spl_token_2022::ID,
    );

    let transfer_instruction = transfer_checked(
        &spl_token_2022::ID,
        &usera_account,
        &fungible_mint,
        &userb_account,
        &usera.pubkey(),
        &[&usera.pubkey()],
        amount,
        DECIMALS,
    )
    .unwrap();

    execute(context, &usera, &[transfer_instruction], vec![&usera])
        .await
        .unwrap();

    println!("accomplished transfer {amount}");

    Ok(())
}

async fn test_bind(
    context: &mut ProgramTestContext,
    source: Pubkey,
    x404_state: Pubkey,
    owner_store: Pubkey,
    bind_mint: Pubkey,
    user: &Keypair,
    fungible_mint: Pubkey,
    number: u64,
) -> Result<(), BanksClientError> {
    println!("start to bind NFT-{number} for {source}");
    let bind_holder =
        get_associated_token_address_with_program_id(&x404_state, &bind_mint, &spl_token_2022::ID);
    let bind_receiver = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &bind_mint,
        &spl_token_2022::ID,
    );

    let fungible_account = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &fungible_mint,
        &spl_token_2022::ID,
    );

    let bind_instruction = x404::instructions::bind(
        number,
        source,
        x404_state,
        owner_store,
        bind_mint,
        bind_holder,
        bind_receiver,
        fungible_mint,
        fungible_account,
        user.pubkey(),
    );

    execute(context, &user, &[bind_instruction], vec![&user])
        .await
        .unwrap();
    println!("accomplished bind NFT-{number} for {source}");
    Ok(())
}

async fn test_unbind(
    context: &mut ProgramTestContext,
    source: Pubkey,
    x404_state: Pubkey,
    owner_store: Pubkey,
    bind_mint: Pubkey,
    user: &Keypair,
    fungible_mint: Pubkey,
    number: u64,
) -> Result<(), BanksClientError> {
    println!("start to unbind {bind_mint} for {source}");
    let unbind_holder = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &bind_mint,
        &spl_token_2022::ID,
    );
    let unbind_receiver =
        get_associated_token_address_with_program_id(&x404_state, &bind_mint, &spl_token_2022::ID);

    let fungible_account = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &fungible_mint,
        &spl_token_2022::ID,
    );

    let unbind_instruction = x404::instructions::unbind(
        number,
        source,
        x404_state,
        owner_store,
        bind_mint,
        unbind_holder,
        unbind_receiver,
        fungible_mint,
        fungible_account,
        user.pubkey(),
    );

    execute(context, &user, &[unbind_instruction], vec![&user])
        .await
        .unwrap();

    println!("accomplish unbind {bind_mint} for {source}");
    Ok(())
}

async fn test_redeem(
    context: &mut ProgramTestContext,
    source: Pubkey,
    x404_state: Pubkey,
    owner_store: Pubkey,
    withdraw_mint: Pubkey,
    user: &Keypair,
    fungible_mint: Pubkey,
    old_owner: Pubkey,
) -> Result<(), BanksClientError> {
    println!("start to redeem {withdraw_mint}");
    let withdraw_holder =
        get_associated_token_address_with_program_id(&x404_state, &withdraw_mint, &spl_token::ID);

    let withdraw_receiver = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &withdraw_mint,
        &spl_token::ID,
    );

    let nft_bank = add_pda(&[b"nft_bank".as_ref(), withdraw_mint.as_ref()], ID);

    let user_account = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &fungible_mint,
        &spl_token_2022::ID,
    );

    let original_owner_account = get_associated_token_address_with_program_id(
        &old_owner,
        &fungible_mint,
        &spl_token_2022::ID,
    );

    let deposit_instructiona = x404::instructions::redeem_spl_nft(
        source,
        x404_state,
        owner_store,
        withdraw_mint,
        withdraw_holder,
        withdraw_receiver,
        nft_bank,
        original_owner_account,
        fungible_mint,
        user_account,
        user.pubkey(),
    );
    execute(context, user, &[deposit_instructiona], vec![user])
        .await
        .unwrap();

    println!("accomplished redeem {withdraw_mint}");
    Ok(())
}
