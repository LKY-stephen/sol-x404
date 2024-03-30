use anchor_lang::{AccountDeserialize, Id, Key};
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id, metadata, token_2022::Token2022,
};
use solana_program::instruction::Instruction;
use solana_program_test::{tokio, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::{AccountSharedData, ReadableAccount},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

use x404::ID;

#[tokio::test]
async fn test_program() {
    let mut validator = ProgramTest::default();
    validator.add_program("X404", ID, None);
    validator.add_program("metaplex_token_metadata_program", metadata::ID, None);

    let owner = add_account(&mut validator);
    let hub_state = add_pda(&[b"hub".as_ref()], ID);
    let source = add_pda(&[b"test_mint".as_ref()], Token2022::id());
    let x404_state = add_pda(&[b"state".as_ref(), source.as_ref()], ID);
    let nft_mint = add_pda(&[b"nft_mint".as_ref(), x404_state.as_ref()], ID);

    let nft_token =
        get_associated_token_address_with_program_id(&nft_mint, &nft_mint, &Token2022::id());
    let fungible_mint = add_pda(&[b"fungible_mint".as_ref(), x404_state.as_ref()], ID);

    let mut context = validator.start_with_context().await;

    // initiate the hub

    let init_instruction = x404::instructions::initialize(hub_state, owner.pubkey());

    execute(&mut context, &owner, &[init_instruction], vec![&owner])
        .await
        .unwrap();

    println!("check hub");
    let read_hub = context
        .banks_client
        .get_account(hub_state)
        .await
        .unwrap()
        .unwrap();

    let hub_data = x404::state::X404Hub::try_deserialize(&mut read_hub.data().as_ref()).unwrap();

    assert_eq!(hub_data.manager, owner.pubkey());
    assert_eq!(hub_data.embergency_close, false);

    // create x404

    let redeemfee = 100;
    let redeem_max_deadline = 100;
    let decimals = 2;

    let init_x404_instruction = x404::instructions::create_x404(
        redeemfee,
        redeem_max_deadline,
        decimals,
        hub_state,
        source,
        x404_state,
        nft_mint,
        fungible_mint,
        owner.pubkey(),
        1000,
    );

    execute(&mut context, &owner, &[init_x404_instruction], vec![&owner])
        .await
        .unwrap();

    println!("check state");
    let read_state = context
        .banks_client
        .get_account(x404_state)
        .await
        .unwrap()
        .unwrap();

    let state_data =
        x404::state::X404State::try_deserialize(&mut read_state.data().as_ref()).unwrap();

    assert_eq!(state_data.nft_mint, nft_mint.key());
    assert_eq!(state_data.fungible_mint, fungible_mint.key());
    assert_eq!(state_data.owner, owner.pubkey());
    assert_eq!(state_data.redeem_fee, redeemfee);
    assert_eq!(state_data.redeem_max_deadline, redeem_max_deadline);
    assert_eq!(state_data.decimal, decimals);

    // create collection

    let init_collection_instruction = x404::instructions::create_collection(
        "test_nft".to_string(),
        "test_nft".to_string(),
        "https://www.google.com/images/branding/googlelogo/1x/googlelogo_light_color_272x92dp.png"
            .to_string(),
        source,
        x404_state,
        nft_mint,
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
}

fn add_account(validator: &mut ProgramTest) -> Keypair {
    let keypair = Keypair::new();
    let account = AccountSharedData::new(5_000_000_000, 0, &solana_sdk::system_program::id());
    validator.add_account(keypair.pubkey(), account.into());
    keypair
}

fn add_pda(seeds: &[&[u8]], id: Pubkey) -> Pubkey {
    Pubkey::find_program_address(seeds, &id).0
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
