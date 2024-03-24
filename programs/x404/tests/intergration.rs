use anchor_lang::{AccountDeserialize, Id};
use anchor_spl::{associated_token::get_associated_token_address, token::Token};
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

    let owner = add_account(&mut validator);
    let hub_state = add_pda(&[b"hub".as_ref()], ID);
    let source = add_pda(&[b"test_mint".as_ref()], Token::id());
    let x404_state = add_pda(&[b"404_contract".as_ref(), source.as_ref()], ID);
    let nft_mint = add_pda(
        &[b"404_contract_nft_mint".as_ref(), x404_state.as_ref()],
        ID,
    );

    let nft_meta = add_pda(
        &[
            b"metadata",
            anchor_spl::metadata::ID.as_ref(),
            nft_mint.as_ref(),
        ],
        anchor_spl::metadata::ID,
    );
    let nft_token = get_associated_token_address(&nft_mint, &nft_mint);
    let fungible_mint = add_pda(
        &[b"404_contract_fungible_mint".as_ref(), x404_state.as_ref()],
        ID,
    );

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

    let create_instruction = x404::instructions::create_x404(
        "test_nft".to_string(),
        "test_nft".to_string(),
        100,
        100,
        2,
        "https://www.google.com/images/branding/googlelogo/1x/googlelogo_light_color_272x92dp.png"
            .to_string(),
        hub_state,
        source,
        x404_state,
        nft_meta,
        nft_mint,
        nft_token,
        fungible_mint,
        owner.pubkey(),
    );
    execute(&mut context, &owner, &[create_instruction], vec![&owner])
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
