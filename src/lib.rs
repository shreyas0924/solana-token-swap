use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    msg,
    program_pack::Pack,
};
use spl_token::state::Account as TokenAccount;

pub mod processor {
    use super::*;

    pub fn process_instruction(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let user_account = next_account_info(accounts_iter)?;
        let from_token_account = next_account_info(accounts_iter)?;
        let to_token_account = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;

        if !user_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (instruction_index, rest) = instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
        if *instruction_index != 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let (amount_bytes, exchange_rate_bytes) = rest.split_at(8);
        let amount = u64::from_le_bytes(amount_bytes.try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let exchange_rate = u64::from_le_bytes(exchange_rate_bytes.try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        let from_token_account_data = TokenAccount::unpack(&from_token_account.data.borrow())?;
        let _to_token_account_data = TokenAccount::unpack(&to_token_account.data.borrow())?;

        if from_token_account_data.amount < amount {
            return Err(ProgramError::InsufficientFunds);
        }

        let _to_amount = amount * exchange_rate;

        let transfer_instruction = spl_token::instruction::transfer(
            token_program.key,
            from_token_account.key,
            to_token_account.key,
            user_account.key,
            &[],
            amount,
        )?;

        invoke(
            &transfer_instruction,
            &[
                user_account.clone(),
                from_token_account.clone(),
                to_token_account.clone(),
                token_program.clone(),
            ],
        )?;

        msg!("Swap complete!");

        Ok(())
    }
}

// Export the process_instruction function
pub use processor::process_instruction;
