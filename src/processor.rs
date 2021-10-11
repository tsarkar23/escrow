use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use std::convert::TryInto;

use crate::instruction::EscrowInstruction;

/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct EscrowData {
    xval: u64,
    yval: u64,
    pass: [u8; 8],
    a_pub_key: Pubkey,
    b_pub_key: Pubkey,
    mint_x_pub_key: Pubkey,
    mint_y_pub_key: Pubkey,
    vault_x_pub_key: Pubkey,
    vault_y_pub_key: Pubkey,
    init_deposit_status: u64,
    is_a_withdrawed: u8,
    is_b_withdrawed: u8,
}

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EscrowInstruction::try_from_slice(instruction_data)?;

        match instruction {
            EscrowInstruction::InitEscrow { amounts } => {
                msg!("Instruction: InitEscrow");
                Self::process_init_escrow(accounts, amounts, program_id)
            }
            EscrowInstruction::Deposit => {
                msg!("Instruction: Deposit");
                Self::process_deposit(accounts)
            }
            EscrowInstruction::Withdrawal => {
                msg!("Instruction: Withdrawal");
                Self::process_withdrawal(accounts, program_id)
            }
        }
    }

    pub fn process_withdrawal(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let escrow_info = next_account_info(account_info_iter)?; // mint  public address
        let taker_token_info = next_account_info(account_info_iter)?;
        let vault_info = next_account_info(account_info_iter)?; // mint  public address
        let taker_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?; // token_program_id

        let escrow_data = EscrowData::try_from_slice(&escrow_info.data.borrow())?;
        
        let amount;

        if escrow_data.init_deposit_status <= 2 {
            msg!("Hey it is not the time to withdraw, you may go ahead and cancel to get back your token!");
            return Err(ProgramError::InvalidAccountData);
        }

        if taker_info.key.as_ref() == escrow_data.a_pub_key.as_ref() {
            if escrow_data.is_a_withdrawed != 0 {
                msg!("Already done!");
                return Err(ProgramError::InvalidAccountData);
            }
            amount = escrow_data.yval;
        } else if taker_info.key.as_ref() == escrow_data.b_pub_key.as_ref() {
            if escrow_data.is_b_withdrawed != 0 {
                msg!("Already done!");
                return Err(ProgramError::InvalidAccountData);
            }
            amount = escrow_data.xval;
        } else{
            msg!("Not authorized!");
            return Err(ProgramError::InvalidAccountData);
        };

        let escrow_seeds = &[
            b"escrow",
            escrow_data.pass.as_ref(),
            escrow_data.a_pub_key.as_ref(),
            escrow_data.b_pub_key.as_ref(),
            escrow_data.mint_x_pub_key.as_ref(),
            escrow_data.mint_y_pub_key.as_ref(),
        ];

        // Do the checkings

        withdraw_token(
            program_id,
            escrow_info,
            vault_info,
            taker_token_info,
            token_program_info,
            escrow_seeds,
            amount,
        )?;

        let escrow_data = EscrowData {
            xval: escrow_data.xval,
            yval: escrow_data.yval,
            pass: escrow_data.pass,
            a_pub_key: escrow_data.a_pub_key,
            b_pub_key: escrow_data.b_pub_key,
            mint_x_pub_key: escrow_data.mint_x_pub_key,
            mint_y_pub_key: escrow_data.mint_y_pub_key,
            vault_x_pub_key: escrow_data.vault_x_pub_key,
            vault_y_pub_key: escrow_data.vault_y_pub_key,
            init_deposit_status: escrow_data.init_deposit_status, //0: not initialized, 1: initialized but no one deposited, 2: alice deposited, 3:both deposited
            is_a_withdrawed: if taker_info.key.as_ref() == escrow_data.a_pub_key.as_ref() {
                1
            } else {
                escrow_data.is_a_withdrawed
            },
            is_b_withdrawed: if taker_info.key.as_ref() == escrow_data.b_pub_key.as_ref() {
                1
            } else {
                escrow_data.is_b_withdrawed
            },
        };

        escrow_data.serialize(&mut &mut escrow_info.data.borrow_mut()[..])?;

        //todo: delete escrow if compeletely done!

        Ok(())
    }

    pub fn process_deposit(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let escrow_info = next_account_info(account_info_iter)?; // mint  public address
        let payer_token_info = next_account_info(account_info_iter)?;
        let vault_info = next_account_info(account_info_iter)?; // mint  public address
        let payer_info = next_account_info(account_info_iter)?; // payer_account, is it both public and private key? yeah
        let token_program_info = next_account_info(account_info_iter)?; // token_program_id

        // Todo: Check if token_program_info is the "real" token program info

        let escrow_data = EscrowData::try_from_slice(&escrow_info.data.borrow())?;
        let amount;

        if escrow_data.init_deposit_status == 0 {
            if payer_info.key.as_ref() != escrow_data.a_pub_key.as_ref() {
                msg!("Account has not initialized yet!");
                return Err(ProgramError::InvalidAccountData);
            }
        }

        if escrow_data.init_deposit_status == 1 {
            if payer_info.key.as_ref() != escrow_data.a_pub_key.as_ref() {
                msg!(
                    "Initiator ({}) needs to transfer! But this id want's to pay: {}",
                    escrow_data.a_pub_key,
                    payer_info.key
                );
                return Err(ProgramError::InvalidAccountData);
            }
            amount = escrow_data.xval;
        }else if escrow_data.init_deposit_status == 2 {
            if payer_info.key.as_ref() != escrow_data.b_pub_key.as_ref() {
                msg!("other side should transfer at this moment!");
                return Err(ProgramError::InvalidAccountData);
            }
            amount = escrow_data.yval;
        } else {
            msg!("Hey it is not the time!");
            return Err(ProgramError::InvalidAccountData);
        }

        deposit_token(
            payer_info,
            vault_info,
            payer_token_info,
            token_program_info,
            amount,
        )?;

        let escrow_data = EscrowData {
            xval: escrow_data.xval,
            yval: escrow_data.yval,
            pass: escrow_data.pass,
            a_pub_key: escrow_data.a_pub_key,
            b_pub_key: escrow_data.b_pub_key,
            mint_x_pub_key: escrow_data.mint_x_pub_key,
            mint_y_pub_key: escrow_data.mint_y_pub_key,
            vault_x_pub_key: escrow_data.vault_x_pub_key,
            vault_y_pub_key: escrow_data.vault_y_pub_key,
            init_deposit_status: escrow_data.init_deposit_status + 1, //0: not initialized, 1: initialized but no one deposited, 2: alice deposited, 3:both deposited
            is_a_withdrawed: 0,
            is_b_withdrawed: 0,
        };

        escrow_data.serialize(&mut &mut escrow_info.data.borrow_mut()[..])?;
        Ok(())
    }

    pub fn process_init_escrow(
        accounts: &[AccountInfo],
        amounts: [u64; 3],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let escrow_info = next_account_info(account_info_iter)?; // mint  public address
        let mint_x_info = next_account_info(account_info_iter)?; // mint  public address
        let mint_y_info = next_account_info(account_info_iter)?; // mint  public address
        let vault_x_info = next_account_info(account_info_iter)?; // mint  public address
        let vault_y_info = next_account_info(account_info_iter)?; // mint  public address
        let payer_info = next_account_info(account_info_iter)?; // payer_account, is it both public and private key? yeah
        let alice_info = next_account_info(account_info_iter)?; // payer_account, is it both public and private key? yeah
        let bob_info = next_account_info(account_info_iter)?; // payer_account, is it both public and private key? yeah
        let token_program_info = next_account_info(account_info_iter)?; // token_program_id
        let rent_info = next_account_info(account_info_iter)?; // solana.py from solana.sysvar import SYSVAR_RENT_PUBKEY
        let system_program_info = next_account_info(account_info_iter)?; // system program public key? public_key(1)?

        create_account(
            program_id,
            &vault_x_info.clone(),
            &mint_x_info.clone(),
            &escrow_info.clone(), // account owner info
            &payer_info.clone(),
            &token_program_info.clone(),
            &rent_info.clone(),
            &system_program_info.clone(),
            &[
                b"vault_x",
                amounts[2].to_le_bytes().as_ref(),
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
            ],
            165,
        )?;

        create_account(
            program_id,
            &vault_y_info.clone(),
            &mint_y_info.clone(),
            &escrow_info.clone(), // account owner info
            &payer_info.clone(),
            &token_program_info.clone(),
            &rent_info.clone(),
            &system_program_info.clone(),
            &[
                b"vault_y1",
                amounts[2].to_le_bytes().as_ref(),
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
            ],
            165,
        )?;

        create_escrow_account(
            program_id,
            &escrow_info.clone(),
            &payer_info.clone(),
            &rent_info.clone(),
            &system_program_info.clone(),
            &[
                b"escrow",
                amounts[2].to_le_bytes().as_ref(),
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
            ],
            218+64,
        )?;

        let escrow_data = EscrowData {
            xval: amounts[0],
            yval: amounts[1],
            pass: amounts[2].to_le_bytes(),
            a_pub_key: *alice_info.key,
            b_pub_key: *bob_info.key,
            mint_x_pub_key: *mint_x_info.key,
            mint_y_pub_key: *mint_y_info.key,
            vault_x_pub_key: *vault_x_info.key,
            vault_y_pub_key: *vault_y_info.key,
            init_deposit_status: 1u64, //0: not initialized, 1: initialized but no one deposited, 2: alice deposited, 3:both deposited
            is_a_withdrawed: 0,
            is_b_withdrawed: 0,
        };

        escrow_data.serialize(&mut &mut escrow_info.data.borrow_mut()[..])?;
        Ok(())
    }
}

fn deposit_token<'a>(
    sender_info: &AccountInfo<'a>, //initiator
    vault_info: &AccountInfo<'a>,
    token_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    let transfer_from_sender = spl_token::instruction::transfer(
        token_program_info.key,
        token_info.key,
        &vault_info.key,
        &sender_info.key,    //alice
        &[&sender_info.key], //alice
        amount,
    )?;
    solana_program::program::invoke(
        &transfer_from_sender,
        &[
            token_info.clone(),  //X_a
            sender_info.clone(), //alice
            vault_info.clone(),  //vault_X
            token_program_info.clone(),
        ],
    )?;

    Ok(())
}

fn withdraw_token<'a>(
    program_id: &Pubkey,
    escrow_info: &AccountInfo<'a>,
    vault_info: &AccountInfo<'a>,
    token_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    escrow_seed: &[&[u8]],
    amount: u64,
) -> ProgramResult {
    let (_, escrow_bump_seed) = Pubkey::find_program_address(escrow_seed, program_id);

    let esc_seed = &[
        escrow_seed[0],
        escrow_seed[1],
        escrow_seed[2],
        escrow_seed[3],
        escrow_seed[4],
        &[escrow_bump_seed],
    ];

    let transfer_from_vault = spl_token::instruction::transfer(
        token_program_info.key,
        &vault_info.key,
        token_info.key,
        &escrow_info.key,
        &[&escrow_info.key],
        amount,
    )?;
    solana_program::program::invoke_signed(
        &transfer_from_vault,
        &[
            vault_info.clone(),
            escrow_info.clone(),
            token_info.clone(),
            token_program_info.clone(),
        ],
        &[esc_seed],
    )?;

    Ok(())
}

fn create_account<'a>(
    program_id: &Pubkey,
    vault_info: &AccountInfo<'a>,
    mint_info: &AccountInfo<'a>,
    account_owner_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    vault_seed: &[&[u8]],
    space: u64,
) -> ProgramResult {
    let rent = &Rent::from_account_info(rent_info)?;
    let required_lamports = rent
        .minimum_balance(space.try_into().unwrap())
        .max(1)
        .saturating_sub(vault_info.lamports());

    let (_, bump_seed) = Pubkey::find_program_address(vault_seed, program_id);

    let seed = &[
        vault_seed[0],
        vault_seed[1],
        vault_seed[2],
        vault_seed[3],
        vault_seed[4],
        &[bump_seed],
    ];
    solana_program::program::invoke_signed(
        &system_instruction::create_account(
            payer_info.key,    //from_pubkey
            vault_info.key,    //to_pubkey
            required_lamports, //lamports
            space,             //space
            token_program_info.key,
        ),
        &[
            payer_info.clone(),
            vault_info.clone(),
            system_program_info.clone(),
        ],
        &[seed],
    )?;
    solana_program::program::invoke_signed(
        &spl_token::instruction::initialize_account(
            token_program_info.key,
            vault_info.key,
            mint_info.key,
            account_owner_info.key,
        )?,
        &[
            vault_info.clone(),
            mint_info.clone(),
            account_owner_info.clone(),
            rent_info.clone(),
            token_program_info.clone(),
        ],
        &[seed],
    )?;
    Ok(())
}

fn create_escrow_account<'a>(
    program_id: &Pubkey,
    escrow_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    vault_seed: &[&[u8]],
    space: u64,
) -> ProgramResult {
    let rent = &Rent::from_account_info(rent_info)?;
    let required_lamports = rent
        .minimum_balance(space.try_into().unwrap())
        .max(1)
        .saturating_sub(escrow_info.lamports());
    let (_, bump_seed) = Pubkey::find_program_address(vault_seed, program_id);
    let seed = &[
        vault_seed[0],
        vault_seed[1],
        vault_seed[2],
        vault_seed[3],
        vault_seed[4],
        &[bump_seed],
    ];
    solana_program::program::invoke_signed(
        &system_instruction::create_account(
            payer_info.key,    //from_pubkey
            escrow_info.key,   //to_pubkey
            required_lamports, //lamports
            space,             //space
            program_id,
        ),
        &[
            payer_info.clone(),
            escrow_info.clone(),
            system_program_info.clone(),
        ],
        &[seed],
    )?;
    Ok(())
}
