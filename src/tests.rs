use solana_program::{
    pubkey::Pubkey,
    system_instruction,
};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::{
    instruction as token_instruction,
    state::{Account as TokenAccount, Mint},
};

use crate::processor::process_instruction;

#[tokio::test]
async fn test_token_swap() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "solana_token_swap",
        program_id,
        processor!(process_instruction),
    );

    // Create mint and token accounts
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();
    let token_account_a = Keypair::new();
    let token_account_b = Keypair::new();

    // Start the test environment
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create mints
    create_mint(&mut banks_client, &payer, &recent_blockhash, &mint_a).await;
    create_mint(&mut banks_client, &payer, &recent_blockhash, &mint_b).await;

    // Create token accounts
    create_token_account(&mut banks_client, &payer, &recent_blockhash, &token_account_a, &mint_a.pubkey()).await;
    create_token_account(&mut banks_client, &payer, &recent_blockhash, &token_account_b, &mint_b.pubkey()).await;

    // Mint initial tokens
    mint_tokens(&mut banks_client, &payer, &recent_blockhash, &mint_a.pubkey(), &token_account_a.pubkey(), 1000).await;

    // Perform swap
    let amount = 500;
    let exchange_rate = 2;
    let instruction_data = [0, amount.to_le_bytes(), exchange_rate.to_le_bytes()].concat();

    let transaction = Transaction::new_signed_with_payer(
        &[crate::processor::process_instruction(
            &program_id,
            &[
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(token_account_a.pubkey(), false),
                AccountMeta::new(token_account_b.pubkey(), false),
                AccountMeta::new(spl_token::id(), false),
            ],
            &instruction_data,
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();

    // Check final balances
    let account_a = banks_client.get_account(token_account_a.pubkey()).await.unwrap().unwrap();
    let account_b = banks_client.get_account(token_account_b.pubkey()).await.unwrap().unwrap();
    let token_account_a = TokenAccount::unpack(&account_a.data).unwrap();
    let token_account_b = TokenAccount::unpack(&account_b.data).unwrap();

    assert_eq!(token_account_a.amount, 500);
    assert_eq!(token_account_b.amount, 1000);
}

async fn create_mint(banks_client: &mut BanksClient, payer: &Keypair, recent_blockhash: &Hash, mint: &Keypair) {
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);

    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                mint_rent,
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            token_instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                &payer.pubkey(),
                None,
                0,
            ).unwrap(),
        ],
        Some(&payer.pubkey()),
        &[payer, mint],
        *recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();
}

async fn create_token_account(banks_client: &mut BanksClient, payer: &Keypair, recent_blockhash: &Hash, account: &Keypair, mint: &Pubkey) {
    let rent = banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(TokenAccount::LEN);

    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &account.pubkey(),
                account_rent,
                TokenAccount::LEN as u64,
                &spl_token::id(),
            ),
            token_instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                &payer.pubkey(),
            ).unwrap(),
        ],
        Some(&payer.pubkey()),
        &[payer, account],
        *recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();
}

async fn mint_tokens(banks_client: &mut BanksClient, payer: &Keypair, recent_blockhash: &Hash, mint: &Pubkey, account: &Pubkey, amount: u64) {
    let transaction = Transaction::new_signed_with_payer(
        &[token_instruction::mint_to(
            &spl_token::id(),
            mint,
            account,
            &payer.pubkey(),
            &[],
            amount,
        ).unwrap()],
        Some(&payer.pubkey()),
        &[payer],
        *recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();
}
