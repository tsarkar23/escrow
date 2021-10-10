use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryInto;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    system_instruction,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
    // program_pack::{IsInitialized, Pack},
};

use crate::instruction::EscrowInstruction;

/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct Amounts {
    init_escrow: u8,
    xval: u64,
    yval: u64, 
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct Info {
    xval: u64,
    yval: u64, 
    state: u8,
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
                Self::process_deposit(accounts, program_id)
            }
            EscrowInstruction::Withdrawal => {
                msg!("Instruction: Withdrawal");
                Self::process_withdrawal(accounts,program_id)
            }
        }
    }





    pub fn process_withdrawal(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        msg!("w1");
        let account_info_iter = &mut accounts.iter(); 
        let escrow_info = next_account_info(account_info_iter)?; // mint  public address
        let mint_x_info = next_account_info(account_info_iter)?; // mint  public address
        let mint_y_info = next_account_info(account_info_iter)?; // mint  public address
        let y_a_info = next_account_info(account_info_iter)?;
        let x_b_info = next_account_info(account_info_iter)?;
        let y_b_info = next_account_info(account_info_iter)?;
        let vault_x_info = next_account_info(account_info_iter)?; // mint  public address
        let vault_y_info = next_account_info(account_info_iter)?; // mint  public address
        // let payer_info = next_account_info(account_info_iter)?; // payer_account, is it both public and private key? yeah
        let alice_info = next_account_info(account_info_iter)?; // payer_account, is it both public and private key? yeah
        let bob_info = next_account_info(account_info_iter)?; // payer_account, is it both public and private key? yeah
        let token_program_info = next_account_info(account_info_iter)?; // token_program_id
        // let rent_info = next_account_info(account_info_iter)?; // solana.py from solana.sysvar import SYSVAR_RENT_PUBKEY
        // let system_program_info = next_account_info(account_info_iter)?; // system program public key? public_key(1)?
        // let escrow_program_info = next_account_info(account_info_iter)?; 
    
    
    
        let vault_x_seeds = &[
            b"vault_x",
            alice_info.key.as_ref(),
            bob_info.key.as_ref(),
            mint_x_info.key.as_ref(),
            mint_y_info.key.as_ref(),
        ];
    
        

        let vault_y_seeds = &[
            b"vault_y",
            alice_info.key.as_ref(),
            bob_info.key.as_ref(),
            mint_x_info.key.as_ref(),
            mint_y_info.key.as_ref(),
        ];

    
        msg!("w2");
    
        let escrow_seeds = &[
            b"escrow",
            alice_info.key.as_ref(),
            bob_info.key.as_ref(),
            mint_x_info.key.as_ref(),
            mint_y_info.key.as_ref(),
        ];
        msg!("w3");
        // let (escrow_pub_address, escrow_bump_seed) = Pubkey::find_program_address(escrow_seeds, program_id);
    
        // deposit_token
        msg!("w4");
        let mut info = Info::try_from_slice(&escrow_info.data.borrow())?;

        // let args = Args:try_from_slice(_instruction_data)?;
        msg!("w5");
        let y_amount = info.yval;
        msg!("w6");
        let x_amount = info.xval;

        // Do the checkings

        // if x_val=info[0]
        // if the addresses are the same
        // if the state = 1

        msg!("x-amount {}", x_amount);
        msg!("y-amount {}", y_amount);
        msg!("w7");
        deposit_token(program_id, bob_info,   escrow_info,vault_y_info,y_b_info,token_program_info,vault_y_seeds,escrow_seeds, y_amount)?;
        msg!("w8");
        withdraw_token(program_id, escrow_info, vault_x_info,x_b_info,token_program_info,vault_x_seeds,escrow_seeds, x_amount)?;
        msg!("w9");
        withdraw_token(program_id, escrow_info, vault_y_info,y_a_info,token_program_info,vault_y_seeds,escrow_seeds, y_amount)?;   
        
        // let info = Info{xval:info.xval,yval:info.yval,state:2};
        // msg!("Step 14");
        // info.serialize(&mut &mut escrow_info.data.borrow_mut()[..])?;
        msg!("w10");
        Ok(())
    }





    pub fn process_deposit(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        msg!("D - Step 0");
        let account_info_iter = &mut accounts.iter(); 
        let escrow_info = next_account_info(account_info_iter)?; // mint  public address
        let mint_x_info = next_account_info(account_info_iter)?; // mint  public address
        let mint_y_info = next_account_info(account_info_iter)?; // mint  public address
        let x_a_info = next_account_info(account_info_iter)?;
        let vault_x_info = next_account_info(account_info_iter)?; // mint  public address
        // let vault_y_info = next_account_info(account_info_iter)?; // mint  public address
        let alice_info = next_account_info(account_info_iter)?; // payer_account, is it both public and private key? yeah
        let bob_info = next_account_info(account_info_iter)?; // payer_account, is it both public and private key? yeah
        let token_program_info = next_account_info(account_info_iter)?; // token_program_id
        // let rent_info = next_account_info(account_info_iter)?; // solana.py from solana.sysvar import SYSVAR_RENT_PUBKEY
        // let system_program_info = next_account_info(account_info_iter)?; // system program public key? public_key(1)?
        // let escrow_program_info = next_account_info(account_info_iter)?; 
    
    
        msg!("D -Step 1");
    
        let vault_x_seeds = &[
            b"vault_x",
            alice_info.key.as_ref(),
            bob_info.key.as_ref(),
            mint_x_info.key.as_ref(),
            mint_y_info.key.as_ref(),
        ];
    
        
    
        msg!("D - Step 2");
    
        let escrow_seeds = &[
            b"escrow",
            alice_info.key.as_ref(),
            bob_info.key.as_ref(),
            mint_x_info.key.as_ref(),
            mint_y_info.key.as_ref(),
        ];

        msg!("D - Step 3");
        
        // let (escrow_pub_address, escrow_bump_seed) = Pubkey::find_program_address(escrow_seeds, program_id);
    
        // deposit_token
    
        let mut info = Info::try_from_slice(&escrow_info.data.borrow())?;


        msg!("D - Step 4");
        // let args = Args:try_from_slice(_instruction_data)?;

        let amount = info.xval;

        msg!("D - Step 5");

        // Do the checkings

        // if x_val=info[0]
        // if the addresses are the same
        // if the state = 1
        
        deposit_token(
            program_id,
            alice_info,
            escrow_info,
            vault_x_info,
            x_a_info,
            token_program_info,
            vault_x_seeds,
            escrow_seeds,
            amount,
        )?;

        msg!("D - Step 6");

    
        
        let info = Info{xval:info.xval,yval:info.yval,state:2};
        msg!("D - Step 7");
        info.serialize(&mut &mut escrow_info.data.borrow_mut()[..])?;
        msg!("D - Step 8");

        Ok(())
    }










    pub fn process_init_escrow(
        accounts: &[AccountInfo],
        amounts: [u64; 2],
        program_id: &Pubkey,
    ) -> ProgramResult {
        msg!("Step 0");

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
        // let escrow_program_info = next_account_info(account_info_iter)?; 
    
    
        msg!("Step 1");
    
        let vault_x_seeds = &[
            b"vault_x",
            alice_info.key.as_ref(),
            bob_info.key.as_ref(),
            mint_x_info.key.as_ref(),
            mint_y_info.key.as_ref(),
        ];
    
        msg!("Step 2");
    
        let vault_y_seeds = &[
            b"vault_y",
            alice_info.key.as_ref(),
            bob_info.key.as_ref(),
            mint_x_info.key.as_ref(),
            mint_y_info.key.as_ref(),
        ];
    
    
        msg!("Step 3");
    
        let escrow_seeds = &[
            b"escrow",
            alice_info.key.as_ref(),
            bob_info.key.as_ref(),
            mint_x_info.key.as_ref(),
            mint_y_info.key.as_ref(),
        ];
        
    
        msg!("Step 7");
    
    
        create_account(program_id, 
            &vault_x_info.clone(),
            &mint_x_info.clone(),
            &escrow_info.clone(), // account owner info
            &payer_info.clone(),
            &token_program_info.clone(),
            &rent_info.clone(),
            &system_program_info.clone(),
            &[
                b"vault_x",
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
            ],
            165,
        )?;
    
        msg!("Step 8");
    
        create_account(program_id, 
            &vault_y_info.clone(),
            &mint_y_info.clone(),
            &escrow_info.clone(), // account owner info
            &payer_info.clone(),
            &token_program_info.clone(),
            &rent_info.clone(),
            &system_program_info.clone(),
            &[
                b"vault_y",
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
            ],
            165,
        )?;

        // msg!("The vaults are created!");

        // make escrow account

        create_escrow_account(program_id,
            &escrow_info.clone(),
            &payer_info.clone(),
            &rent_info.clone(),
            &system_program_info.clone(),
            &[
                b"escrow",
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
            ],
            17,
        )?;

        msg!("The escrow is created!");


         
        let info = Info{xval:amounts[0],yval:amounts[1],state:1};
        msg!("Step 14");
        info.serialize(&mut &mut escrow_info.data.borrow_mut()[..])?;
        msg!("Step 15");

        Ok(())
    }
}








fn deposit_token<'a>(
    program_id: &Pubkey,
    sender_info: &AccountInfo<'a>, //initiator
    escrow_info: &AccountInfo<'a>,
    vault_info: &AccountInfo<'a>,
    token_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    vault_seed: &[&[u8]],
    escrow_seed: &[&[u8]],
    amount: u64
    ) -> ProgramResult {

    msg!("Step d0");


    let (escrow_pub_address, escrow_bump_seed) = Pubkey::find_program_address(escrow_seed, program_id);
    let (valut_x_pub_address, vault_bump_seed) = Pubkey::find_program_address(vault_seed, program_id);
    
    msg!("Step d1");


    msg!("Step d3");
    let transfer_from_sender = spl_token::instruction::transfer(
        token_program_info.key,
        token_info.key,
        &vault_info.key,
        &sender_info.key, //alice
        &[&sender_info.key], //alice
        amount,
    )?;
    msg!("Step d4");
    solana_program::program::invoke(
        &transfer_from_sender,
        &[
            token_info.clone(),//X_a
            sender_info.clone(),//alice
            vault_info.clone(),//vault_X
            token_program_info.clone(),
        ]
        )?;

    msg!("Step d5");
Ok(())
}







fn withdraw_token<'a>(
    program_id: &Pubkey,
    escrow_info: &AccountInfo<'a>,
    vault_info: &AccountInfo<'a>,
    token_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    vault_seed: &[&[u8]],
    escrow_seed: &[&[u8]],
    amount: u64
    ) -> ProgramResult {

    msg!("Step z1");
    let (escrow_pub_address, escrow_bump_seed) = Pubkey::find_program_address(escrow_seed, program_id);
    let (valut_x_pub_address, vault_x_bump_seed) = Pubkey::find_program_address(vault_seed, program_id);
    
    msg!("Step z2");
    let esc_seed = &[
        escrow_seed[0],
        escrow_seed[1],
        escrow_seed[2],
        escrow_seed[3],
        escrow_seed[4],
        &[escrow_bump_seed],
    ];

    // let x_seed = &[
    //     vault_seed[0],
    //     vault_seed[1],
    //     vault_seed[2],
    //     vault_seed[3],
    //     vault_seed[4],
    //     &[vault_x_bump_seed],
    // ];

    msg!("Step z3");
    let transfer_from_vault = spl_token::instruction::transfer(
        token_program_info.key,
        &vault_info.key,
        token_info.key,
        &escrow_info.key,
        &[&escrow_info.key],
        amount,
    )?;
    msg!("Step z4");
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

    msg!("Step f");
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
    msg!("Step 9");
    
    // let rent: Rent = bincode::deserialize(&rent_info.data.borrow()).map_err(|_| ProgramError::InvalidArgument)?;
    let rent = &Rent::from_account_info(rent_info)?;
    // let space: u64 =  165; // spl_token::state::Account::LEN;
    let required_lamports = rent
        .minimum_balance(space.try_into().unwrap())
        .max(1)
        .saturating_sub(vault_info.lamports());

    
    // let mut seed = &[vault_seeds];

    let (pub_address, bump_seed) = Pubkey::find_program_address(vault_seed, program_id);

    msg!("Step 10");
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
            payer_info.key, //from_pubkey
            vault_info.key, //to_pubkey
            required_lamports, //lamports
            space, //space
        token_program_info.key,
        ),
        &[payer_info.clone(), vault_info.clone(), system_program_info.clone()],
        &[seed],
    )?;
    msg!("Step 11 - here");
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
    msg!("Step 12");
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
    msg!("Step a");
    let rent = &Rent::from_account_info(rent_info)?;
    msg!("Step b");
    let required_lamports = rent
        .minimum_balance(space.try_into().unwrap())
        .max(1)
        .saturating_sub(escrow_info.lamports());
    msg!("Step c");
    let (pub_address, bump_seed) = Pubkey::find_program_address(vault_seed, program_id);
    msg!("Step d");
    let seed = &[
        vault_seed[0],
        vault_seed[1],
        vault_seed[2],
        vault_seed[3],
        vault_seed[4],
        &[bump_seed],
    ];
    msg!("Step e");
    solana_program::program::invoke_signed(
        &system_instruction::create_account(
            payer_info.key, //from_pubkey
            escrow_info.key, //to_pubkey
            required_lamports, //lamports
            space, //space
            program_id,
        ),
        &[payer_info.clone(), escrow_info.clone(), system_program_info.clone()],
        &[seed],
    )?;
    msg!("Step f");
Ok(())
}
