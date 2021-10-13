use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    program_pack::Pack,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use spl_token::{
    state::Account,
    instruction::initialize_account,
    instruction::transfer
};

use crate::instruction::EscrowInstruction;
use crate::state::{EscrowData, EscrowState};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EscrowInstruction::try_from_slice(instruction_data)?;

        match instruction {
            EscrowInstruction::InitEscrow {
                amount_x,
                amount_y,
                pass,
            } => {
                msg!("Instruction: InitEscrow");
                Self::process_init_escrow(accounts, amount_x, amount_y, pass, program_id)
            }
            EscrowInstruction::Deposit { pass } => {
                msg!("Instruction: Deposit");
                Self::process_deposit(accounts, pass, program_id)
            }
            EscrowInstruction::Withdrawal { pass } => {
                msg!("Instruction: Withdrawal");
                Self::process_withdrawal(accounts, pass, program_id)
            }
        }
    }

    pub fn process_init_escrow(
        accounts: &[AccountInfo],
        size_x: u64,
        size_y: u64,
        pass: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let escrow_info = next_account_info(account_info_iter)?;
        let mint_x_info = next_account_info(account_info_iter)?;
        let mint_y_info = next_account_info(account_info_iter)?;
        let vault_x_info = next_account_info(account_info_iter)?;
        let vault_y_info = next_account_info(account_info_iter)?;
        let payer_info = next_account_info(account_info_iter)?;
        let alice_info = next_account_info(account_info_iter)?;
        let bob_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        msg!("Creating vault for mint x");
        let vault_x_bump = create_vault(
            program_id,
            &vault_x_info,
            &alice_info,
            &bob_info,
            &mint_x_info,
            &mint_y_info,
            &escrow_info, // authority_info
            &payer_info,
            &token_program_info,
            &rent_info,
            &system_program_info,
            pass,
            b"vault_x",
        )?;
        solana_program::program::invoke(
            &spl_token::instruction::initialize_account(
                token_program_info.key,
                vault_x_info.key,
                mint_x_info.key,
                escrow_info.key,
            )?,
            &[
                vault_x_info.clone(),
                mint_x_info.clone(),
                escrow_info.clone(),
                rent_info.clone(),
                token_program_info.clone(),
            ],
        )?;
        msg!("Creating vault for mint y");
        let vault_y_bump = create_vault(
            program_id,
            &vault_y_info,
            &alice_info,
            &bob_info,
            &mint_x_info,
            &mint_y_info,
            &escrow_info, // account owner info
            &payer_info,
            &token_program_info,
            &rent_info,
            &system_program_info,
            pass,
            b"vault_y",
        )?;
        solana_program::program::invoke(
            &spl_token::instruction::initialize_account(
                token_program_info.key,
                vault_y_info.key,
                mint_y_info.key,
                escrow_info.key,
            )?,
            &[
                vault_y_info.clone(),
                mint_y_info.clone(),
                escrow_info.clone(),
                rent_info.clone(),
                token_program_info.clone(),
            ],
        )?;
        msg!("Creating escrow metadata");
        let escrow_seeds = &[
            b"escrow",
            alice_info.key.as_ref(),
            bob_info.key.as_ref(),
            mint_x_info.key.as_ref(),
            mint_y_info.key.as_ref(),
            &pass.to_le_bytes(),
        ];
        let rent = &Rent::from_account_info(rent_info)?;
        let required_lamports = rent
            .minimum_balance(EscrowData::LEN)
            .max(1)
            .saturating_sub(escrow_info.lamports());
        let (_, escrow_bump) = Pubkey::find_program_address(escrow_seeds, program_id);
        let escrow_seeds = &[
            b"escrow",
            alice_info.key.as_ref(),
            bob_info.key.as_ref(),
            mint_x_info.key.as_ref(),
            mint_y_info.key.as_ref(),
            &pass.to_le_bytes(),
            &[escrow_bump],
        ];
        solana_program::program::invoke_signed(
            &system_instruction::create_account(
                payer_info.key,    //from_pubkey
                escrow_info.key,   //to_pubkey
                required_lamports, //lamports
                EscrowData::LEN as u64,      //space
                program_id,
            ),
            &[
                payer_info.clone(),
                escrow_info.clone(),
                system_program_info.clone(),
            ],
            &[escrow_seeds],
        )?;
        let escrow_data = EscrowData {
            size_x,
            size_y,
            pubkey_alice: *alice_info.key,
            pubkey_bob: *bob_info.key,
            pubkey_mint_x: *mint_x_info.key,
            pubkey_mint_y: *mint_y_info.key,
            state: EscrowState::Initialized,
            escrow_bump,
            vault_x_bump,
            vault_y_bump,
        };
        escrow_data.serialize(&mut *escrow_info.data.borrow_mut())?;
        Ok(())
    }

    pub fn process_deposit(accounts: &[AccountInfo], pass: u64, program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let escrow_info = next_account_info(account_info_iter)?; // mint  public address
        let payer_token_info = next_account_info(account_info_iter)?;
        let vault_info = next_account_info(account_info_iter)?; // mint  public address
        let payer_info = next_account_info(account_info_iter)?; // payer_account, is it both public and private key? yeah
        let token_program_info = next_account_info(account_info_iter)?; // token_program_id
        let mut escrow_data = EscrowData::try_from_slice(&escrow_info.data.borrow())?;
        msg!("Validating and chaning state");
        match escrow_data.state {
            EscrowState::Initialized => {
                if *payer_info.key == escrow_data.pubkey_alice {
                    escrow_data.state = EscrowState::DepositAlice;
                } else if *payer_info.key == escrow_data.pubkey_bob {
                    escrow_data.state = EscrowState::DepositBob;
                } else {
                    msg!("Invalid State");
                    return Err(ProgramError::InvalidAccountData);                    
                }
            }
            EscrowState::DepositAlice => {
                if *payer_info.key == escrow_data.pubkey_bob {
                    escrow_data.state = EscrowState::Committed;
                } else {
                    msg!("Invalid State");
                    return Err(ProgramError::InvalidAccountData);                    
                }
            }
            EscrowState::DepositBob => {
                if *payer_info.key == escrow_data.pubkey_alice {
                    escrow_data.state = EscrowState::Committed;
                } else {
                    msg!("Invalid State");
                    return Err(ProgramError::InvalidAccountData);                    
                }
            }
            _ => {
                msg!("Invalid State");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        msg!("Validating account ownership");
        if payer_token_info.owner != token_program_info.key {
            msg!("Invalid Token Account (system account not owned by Token Program)");
            return Err(ProgramError::InvalidAccountData);
        }
        let token_account: Account = Account::unpack_unchecked(&payer_token_info.data.borrow())?;
        if token_account.owner != *payer_info.key {
            msg!("Invalid Token Account (\"User space\" owner mismatch)");
            return Err(ProgramError::InvalidAccountData);
        }

        let (vault_seed, bump_seed, size) =  if *payer_info.key == escrow_data.pubkey_alice {
            if token_account.mint != escrow_data.pubkey_mint_x {
                msg!("Invalid Mint");
                return Err(ProgramError::InvalidAccountData); 
            }
            (b"vault_x", escrow_data.vault_x_bump, escrow_data.size_x)
        } else if *payer_info.key == escrow_data.pubkey_bob {
            if token_account.mint != escrow_data.pubkey_mint_y {
                msg!("Invalid Mint");
                return Err(ProgramError::InvalidAccountData); 
            }
            (b"vault_y", escrow_data.vault_y_bump, escrow_data.size_y)
        } else {
            msg!("Invalid Owner");
            return Err(ProgramError::InvalidAccountData);     
        };
        let seeds = &[
            vault_seed,
            escrow_data.pubkey_alice.as_ref(),
            escrow_data.pubkey_bob.as_ref(),
            escrow_data.pubkey_mint_x.as_ref(),
            escrow_data.pubkey_mint_y.as_ref(),
            &pass.to_le_bytes(),
            &[bump_seed],
        ];
        let vault_pubkey = Pubkey::create_program_address(
            seeds,
            program_id,
        )?;
        if vault_pubkey != *vault_info.key {
            msg!("Vault key mismatch");
            return Err(ProgramError::InvalidAccountData);              
        }
        msg!("Sending transfer");
        solana_program::program::invoke(
            &spl_token::instruction::transfer(
                token_program_info.key,
                payer_token_info.key,
                &vault_info.key,
                &payer_info.key,
                &[],
                size,
            )?,
            &[
                payer_token_info.clone(),
                payer_info.clone(),
                vault_info.clone(),
                token_program_info.clone(),
            ],
        )?;
        escrow_data.serialize(&mut &mut escrow_info.data.borrow_mut()[..])?;
        Ok(())
    }

    pub fn process_withdrawal(
        accounts: &[AccountInfo],
        pass: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let escrow_info = next_account_info(account_info_iter)?; // mint  public address
        let taker_token_info = next_account_info(account_info_iter)?;
        let vault_info = next_account_info(account_info_iter)?; // mint  public address
        let taker_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?; // token_program_id
        msg!("process_withdrawal 1");
        let mut escrow_data = EscrowData::try_from_slice(&escrow_info.data.borrow())?;

        let amount;

        msg!("process_withdrawal 2");
        if escrow_data.init_deposit_status <= 2 {
            msg!("Hey it is not the time to withdraw, you may go ahead and cancel to get back your token!");
            return Err(ProgramError::InvalidAccountData);
        }
        msg!("process_withdrawal 3");
        if taker_info.key.as_ref() == escrow_data.a_pub_key.as_ref() {
            if escrow_data.is_a_withdrawed != 0 {
                msg!("Already done!");
                return Err(ProgramError::InvalidAccountData);
            }
            // let mut account = Account::unpack_from_slice(*taker_token_info.data.borrow())?;
            if taker_token_info.owner.as_ref() != escrow_data.a_pub_key.as_ref() {
                msg!("Dangerous activity, the token is going to some other account!");
                return Err(ProgramError::InvalidAccountData);
            }
            amount = escrow_data.yval;
        } else if taker_info.key.as_ref() == escrow_data.b_pub_key.as_ref() {
            if escrow_data.is_b_withdrawed != 0 {
                msg!("Already done!");
                return Err(ProgramError::InvalidAccountData);
            }
            // let mut account = Account::unpack_from_slice(*taker_token_info.data.borrow())?;
            if taker_token_info.owner.as_ref() != escrow_data.b_pub_key.as_ref() {
                msg!("Dangerous activity, the token is going to some other account!");
                // msg!
                return Err(ProgramError::InvalidAccountData);
            }
            amount = escrow_data.xval;
        } else {
            msg!("Not authorized!");
            return Err(ProgramError::InvalidAccountData);
        };
        msg!("process_withdrawal 4");
        let tmp = pass.to_le_bytes();
        let escrow_seed = &[
            b"escrow",
            tmp.as_ref(),
            escrow_data.a_pub_key.as_ref(),
            escrow_data.b_pub_key.as_ref(),
            escrow_data.mint_x_pub_key.as_ref(),
            escrow_data.mint_y_pub_key.as_ref(),
        ];

        msg!("withdraw_token 1");
        let (_, escrow_bump_seed) = Pubkey::find_program_address(escrow_seed, program_id);
        msg!("withdraw_token 2");
        let esc_seed = &[
            escrow_seed[0],
            escrow_seed[1],
            escrow_seed[2],
            escrow_seed[3],
            escrow_seed[4],
            escrow_seed[5],
            &[escrow_bump_seed],
        ];
        msg!("withdraw_token 3");
        let transfer_from_vault = spl_token::instruction::transfer(
            token_program_info.key,
            &vault_info.key,
            taker_token_info.key,
            &escrow_info.key,
            &[&escrow_info.key],
            amount,
        )?;
        msg!("withdraw_token 4");
        solana_program::program::invoke_signed(
            &transfer_from_vault,
            &[
                vault_info.clone(),
                escrow_info.clone(),
                taker_token_info.clone(),
                token_program_info.clone(),
            ],
            &[esc_seed],
        )?;

        msg!("process_withdrawal 6");

        escrow_data.is_a_withdrawed = if taker_info.key.as_ref() == escrow_data.a_pub_key.as_ref() {
            1
        } else {
            escrow_data.is_a_withdrawed
        };

        escrow_data.is_b_withdrawed = if taker_info.key.as_ref() == escrow_data.b_pub_key.as_ref() {
            1
        } else {
            escrow_data.is_b_withdrawed
        };

        msg!("process_withdrawal 7");
        escrow_data.serialize(&mut &mut escrow_info.data.borrow_mut()[..])?;
        msg!("process_withdrawal 8");
        //todo: delete escrow if compeletely done!

        Ok(())
    }
}

fn create_vault<'a>(
    program_id: &Pubkey,
    vault_info: &AccountInfo<'a>,
    alice_info: &AccountInfo<'a>,
    bob_info: &AccountInfo<'a>,
    mint_x_info: &AccountInfo<'a>,
    mint_y_info: &AccountInfo<'a>,
    authority_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    pass: u64,
    vault_seed: &[u8],
) -> Result<u8, ProgramError> {
    let space= Account::LEN;
    let rent = &Rent::from_account_info(rent_info)?;
    let required_lamports = rent
        .minimum_balance(space)
        .max(1)
        .saturating_sub(vault_info.lamports());
    let seeds = &[
        vault_seed,
        alice_info.key.as_ref(),
        bob_info.key.as_ref(),
        mint_x_info.key.as_ref(),
        mint_y_info.key.as_ref(), 
        &pass.to_le_bytes(),
    ];

    let (_, bump_seed) = Pubkey::find_program_address(seeds, program_id);
    let seeds = &[
        vault_seed,
        alice_info.key.as_ref(),
        bob_info.key.as_ref(),
        mint_x_info.key.as_ref(),
        mint_y_info.key.as_ref(),
        &pass.to_le_bytes(),
        &[bump_seed],
    ];

    solana_program::program::invoke_signed(
        &system_instruction::create_account(
            payer_info.key,    //from_pubkey
            vault_info.key,    //to_pubkey
            required_lamports, //lamports
            space as u64,             //space
            token_program_info.key,
        ),
        &[
            payer_info.clone(),
            vault_info.clone(),
            system_program_info.clone(),
        ],
        &[seeds],
    )?;
    Ok(bump_seed)
}
