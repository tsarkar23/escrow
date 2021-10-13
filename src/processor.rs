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
use std::mem;
use crate::instruction::EscrowInstruction;
use crate::state::EscrowData;

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EscrowInstruction::try_from_slice(instruction_data)?;

        match instruction {
            EscrowInstruction::InitEscrow { amount_x, amount_y, pass } => {
                msg!("Instruction: InitEscrow");
                Self::process_init_escrow(accounts, amount_x, amount_y, pass, program_id)
            }
            EscrowInstruction::Deposit {pass}=> {
                msg!("Instruction: Deposit");
                Self::process_deposit(accounts, pass)
            }
            EscrowInstruction::Withdrawal {pass}=> {
                msg!("Instruction: Withdrawal");
                Self::process_withdrawal(accounts, pass, program_id)
            }
        }
    }

    pub fn process_init_escrow(
        accounts: &[AccountInfo],
        amount_x: u64,
        amount_y: u64,
        pass: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        msg!("process_init_escrow");
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


        // msg!("password {}", pass);
        // msg!("password {:?}", pass.to_le_bytes().as_ref());
        // msg!("alice pub key {:?}", alice_info.key.as_ref());
        // return Err(ProgramError::InvalidAccountData);
        msg!("init: 1");
        create_vault(
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
                pass.to_le_bytes().as_ref(),
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
            ],
            165,
        )?;














        
        msg!("init: 2");
        create_vault(
            program_id,
            &vault_y_info.clone(),
            &mint_y_info.clone(),
            &escrow_info.clone(), // account owner info
            &payer_info.clone(),
            &token_program_info.clone(),
            &rent_info.clone(),
            &system_program_info.clone(),
            &[
                b"vault_y",
                pass.to_le_bytes().as_ref(),
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
            ],
            165,
        )?;
        msg!("init: 3");
        let escrow_size:u64 = (mem::size_of::<EscrowData>()).try_into().unwrap();
        msg!("init: 3 {:?}", escrow_size);




        // create_escrow_account(
        //     program_id,
        //     &escrow_info.clone(),
        //     &payer_info.clone(),
        //     &rent_info.clone(),
        //     &system_program_info.clone(),
        //     &[
        //         b"escrow",
        //         pass.to_le_bytes().as_ref(),
        //         alice_info.key.as_ref(),
        //         bob_info.key.as_ref(),
        //         mint_x_info.key.as_ref(),
        //         mint_y_info.key.as_ref(),
        //     ],
        //     218, // escrow_size
        // )?;
        let tmp = pass.to_le_bytes();
        let escrow_seed = [
            b"escrow",
            tmp.as_ref(),
            alice_info.key.as_ref(),
            bob_info.key.as_ref(),
            mint_x_info.key.as_ref(),
            mint_y_info.key.as_ref(),
        ];
        let escrow_space = 218;
        let rent = &Rent::from_account_info(rent_info)?;
        let required_lamports = rent
            .minimum_balance(escrow_space.try_into().unwrap())
            .max(1)
            .saturating_sub(escrow_info.lamports());
        let (_, bump_seed) = Pubkey::find_program_address(&escrow_seed, program_id);
        let seed = &[
            escrow_seed[0],
            escrow_seed[1],
            escrow_seed[2],
            escrow_seed[3],
            escrow_seed[4],
            escrow_seed[5],
            &[bump_seed],
        ];
        solana_program::program::invoke_signed(
            &system_instruction::create_account(
                payer_info.key,    //from_pubkey
                escrow_info.key,   //to_pubkey
                required_lamports, //lamports
                escrow_space,             //space
                program_id,
            ),
            &[
                payer_info.clone(),
                escrow_info.clone(),
                system_program_info.clone(),
            ],
            &[seed],
        )?;


        





















        msg!("init: 4");
        let escrow_data = EscrowData {
            xval: amount_x,
            yval: amount_y,
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
        msg!("init: 5");
        escrow_data.serialize(&mut &mut escrow_info.data.borrow_mut()[..])?; //write escrow_data into escrow_info.data
        msg!("init: 6");
        Ok(())
    }




    pub fn process_deposit(
        accounts: &[AccountInfo],
        pass: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let escrow_info = next_account_info(account_info_iter)?; // mint  public address
        let payer_token_info = next_account_info(account_info_iter)?;
        let vault_info = next_account_info(account_info_iter)?; // mint  public address
        let payer_info = next_account_info(account_info_iter)?; // payer_account, is it both public and private key? yeah
        let token_program_info = next_account_info(account_info_iter)?; // token_program_id

        // Todo: Check if token_program_info is the "real" token program info
        msg!("process_deposit started here!");
        let escrow_size:u64 = (mem::size_of::<EscrowData>()).try_into().unwrap();
        msg!("process_deposit 1: {:?}", escrow_size);
        let mut escrow_data = EscrowData::try_from_slice(&escrow_info.data.borrow())?;
        msg!("process_deposit 1");
        let amount;
        msg!("process_deposit 2");
        if escrow_data.init_deposit_status == 0 {
            if payer_info.key.as_ref() != escrow_data.a_pub_key.as_ref() {
                msg!("Account has not initialized yet!");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        msg!("process_deposit 3");
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
        msg!("process_deposit 4");
        let transfer_from_sender = spl_token::instruction::transfer(
            token_program_info.key,
            payer_token_info.key,
            &vault_info.key,
            &payer_info.key,    //alice
            &[&payer_info.key], //alice
            amount,
        )?;
        msg!("deposit_token 2");
        solana_program::program::invoke(
            &transfer_from_sender,
            &[
                payer_token_info.clone(),  //X_a
                payer_info.clone(), //alice
                vault_info.clone(),  //vault_X
                token_program_info.clone(),
            ],
        )?;
        msg!("deposit_token 3");

        escrow_data.init_deposit_status = escrow_data.init_deposit_status + 1;
        msg!("process_deposit 6");
        escrow_data.serialize(&mut &mut escrow_info.data.borrow_mut()[..])?;
        Ok(())
    }


    pub fn process_withdrawal(
        accounts: &[AccountInfo],
        pass: u64,
        program_id: &Pubkey) -> ProgramResult {
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
    msg!("create_account: 4 - seed");
    let seed = &[
        vault_seed[0],
        vault_seed[1],
        vault_seed[2],
        vault_seed[3],
        vault_seed[4],
        vault_seed[5],
        &[bump_seed],
    ];
    msg!("create_account:5 - seed {:?}", seed);
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
    msg!("create_account: 6 - seed");
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

// fn create_escrow_account<'a>(
//     program_id: &Pubkey,
//     escrow_info: &AccountInfo<'a>,
//     payer_info: &AccountInfo<'a>,
//     rent_info: &AccountInfo<'a>,
//     system_program_info: &AccountInfo<'a>,
//     escrow_seed: &[&[u8]],
//     space: u64,
// ) -> ProgramResult {

//     let rent = &Rent::from_account_info(rent_info)?;
//     let required_lamports = rent
//         .minimum_balance(space.try_into().unwrap())
//         .max(1)
//         .saturating_sub(escrow_info.lamports());
//     let (_, bump_seed) = Pubkey::find_program_address(escrow_seed, program_id);
//     let seed = &[
//         escrow_seed[0],
//         escrow_seed[1],
//         escrow_seed[2],
//         escrow_seed[3],
//         escrow_seed[4],
//         escrow_seed[5],
//         &[bump_seed],
//     ];
//     solana_program::program::invoke_signed(
//         &system_instruction::create_account(
//             payer_info.key,    //from_pubkey
//             escrow_info.key,   //to_pubkey
//             required_lamports, //lamports
//             space,             //space
//             program_id,
//         ),
//         &[
//             payer_info.clone(),
//             escrow_info.clone(),
//             system_program_info.clone(),
//         ],
//         &[seed],
//     )?;
//     Ok(())
// }

