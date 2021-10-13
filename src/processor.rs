use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use spl_token::{instruction::initialize_account, instruction::transfer, state::Account};

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
        pass: [u8; 32],
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

        let escrow_bump = if escrow_info.data_len() == 0 {
            msg!("Creating escrow metadata");
            let escrow_seeds = &[
                b"escrow",
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
                pass.as_ref(), //.to_le_bytes(),
            ];
            let rent = &Rent::from_account_info(rent_info)?;
            let required_lamports = rent
                .minimum_balance(EscrowData::LEN)
                .max(1)
                .saturating_sub(escrow_info.lamports());
            let (_, bump) = Pubkey::find_program_address(escrow_seeds, program_id);
            let escrow_seeds = &[
                b"escrow",
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
                pass.as_ref(),
                &[bump],
            ];
            solana_program::program::invoke_signed(
                &system_instruction::create_account(
                    payer_info.key,         //from_pubkey
                    escrow_info.key,        //to_pubkey
                    required_lamports,      //lamports
                    EscrowData::LEN as u64, //space
                    program_id,
                ),
                &[
                    payer_info.clone(),
                    escrow_info.clone(),
                    system_program_info.clone(),
                ],
                &[escrow_seeds],
            )?;
            bump
        } else {
            let escrow_seeds = &[
                b"escrow",
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
                pass.as_ref(), // .to_le_bytes(),
            ];
            let (_, bump) = Pubkey::find_program_address(escrow_seeds, program_id);
            bump
        };
        let vault_x_bump = if vault_x_info.data_len() == 0 {
            msg!("Creating vault for mint x");
            let bump = create_vault(
                program_id,
                &vault_x_info,
                &alice_info,
                &bob_info,
                &mint_x_info,
                &mint_y_info,
                &payer_info,
                &token_program_info,
                &rent_info,
                &system_program_info,
                pass,
                b"vault_x",
            )?;
            solana_program::program::invoke(
                &initialize_account(
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
            bump
        } else {
            let seeds = &[
                b"vault_x",
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
                pass.as_ref(), //.to_le_bytes(),
            ];
            let (_, bump) = Pubkey::find_program_address(seeds, program_id);
            bump
        };
        let vault_y_bump = if vault_y_info.data_len() == 0 {
            msg!("Creating vault for mint y");
            let bump = create_vault(
                program_id,
                &vault_y_info,
                &alice_info,
                &bob_info,
                &mint_x_info,
                &mint_y_info,
                &payer_info,
                &token_program_info,
                &rent_info,
                &system_program_info,
                pass,
                b"vault_y",
            )?;
            solana_program::program::invoke(
                &initialize_account(
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
            bump
        } else {
            let seeds = &[
                b"vault_y",
                alice_info.key.as_ref(),
                bob_info.key.as_ref(),
                mint_x_info.key.as_ref(),
                mint_y_info.key.as_ref(),
                pass.as_ref(),
            ];
            let (_, bump) = Pubkey::find_program_address(seeds, program_id);
            bump
        };

        let escrow_data = EscrowData::try_from_slice(&escrow_info.data.borrow())?;
        if escrow_data.state != EscrowState::Uninitialized {
            msg!("Trying reinitialize an existing escrow");
            return Err(ProgramError::InvalidAccountData.into());
        }

        EscrowData {
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
        }
        .serialize(&mut *escrow_info.data.borrow_mut())?;
        Ok(())
    }

    pub fn process_deposit(
        accounts: &[AccountInfo],
        pass: [u8;32],
        program_id: &Pubkey,
    ) -> ProgramResult {
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

        let (vault_seed, bump_seed, size) = if *payer_info.key == escrow_data.pubkey_alice {
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
            pass.as_ref(), // .to_le_bytes(),
            &[bump_seed],
        ];
        let vault_pubkey = Pubkey::create_program_address(seeds, program_id)?;
        if vault_pubkey != *vault_info.key {
            msg!("Vault key mismatch");
            return Err(ProgramError::InvalidAccountData);
        }
        msg!("Validating escrow data");
        let escrow_seeds = &[
            b"escrow",
            escrow_data.pubkey_alice.as_ref(),
            escrow_data.pubkey_bob.as_ref(),
            escrow_data.pubkey_mint_x.as_ref(),
            escrow_data.pubkey_mint_y.as_ref(),
            pass.as_ref(), // .to_le_bytes(),
            &[escrow_data.escrow_bump],
        ];
        let escrow_key = Pubkey::create_program_address(escrow_seeds, program_id)?;
        if escrow_key != *escrow_info.key {
            msg!("Escrow key mismatch");
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
        pass: [u8; 32],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let escrow_info = next_account_info(account_info_iter)?;
        let taker_token_info = next_account_info(account_info_iter)?;
        let vault_info = next_account_info(account_info_iter)?;
        let taker_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        msg!("process_withdrawal 1");
        let mut escrow_data = EscrowData::try_from_slice(&escrow_info.data.borrow())?;

        let withdraw_mint = match escrow_data.state {
            EscrowState::Committed => {
                if *taker_info.key == escrow_data.pubkey_alice {
                    escrow_data.state = EscrowState::WithdrawAlice;
                    escrow_data.pubkey_mint_y
                } else if *taker_info.key == escrow_data.pubkey_bob {
                    escrow_data.state = EscrowState::WithdrawBob;
                    escrow_data.pubkey_mint_x
                } else {
                    msg!("Invalid State");
                    return Err(ProgramError::InvalidAccountData);
                }
            }
            EscrowState::WithdrawAlice => {
                if *taker_info.key == escrow_data.pubkey_bob {
                    escrow_data.state = EscrowState::Uninitialized;
                    escrow_data.pubkey_mint_x
                } else {
                    msg!("Invalid State");
                    return Err(ProgramError::InvalidAccountData);
                }
            }
            EscrowState::WithdrawBob => {
                if *taker_info.key == escrow_data.pubkey_alice {
                    escrow_data.state = EscrowState::Uninitialized;
                    escrow_data.pubkey_mint_y
                } else {
                    msg!("Invalid State");
                    return Err(ProgramError::InvalidAccountData);
                }
            }
            EscrowState::DepositAlice => {
                if *taker_info.key == escrow_data.pubkey_alice {
                    escrow_data.state = EscrowState::Initialized;
                    escrow_data.pubkey_mint_x
                } else {
                    msg!("Invalid State");
                    return Err(ProgramError::InvalidAccountData);
                }
            }
            EscrowState::DepositBob => {
                if *taker_info.key == escrow_data.pubkey_alice {
                    escrow_data.state = EscrowState::Initialized;
                    escrow_data.pubkey_mint_y
                } else {
                    msg!("Invalid State");
                    return Err(ProgramError::InvalidAccountData);
                }
            }
            _ => {
                msg!("Invalid State");
                return Err(ProgramError::InvalidAccountData);
            }
        };

        msg!("Validating account ownership");
        if taker_token_info.owner != token_program_info.key {
            msg!("Invalid Token Account (system account not owned by Token Program)");
            return Err(ProgramError::InvalidAccountData);
        }
        let token_account: Account = Account::unpack_unchecked(&taker_token_info.data.borrow())?;
        if token_account.owner != *taker_info.key {
            msg!("Invalid Token Account (\"User space\" owner mismatch)");
            return Err(ProgramError::InvalidAccountData);
        }

        if token_account.mint != withdraw_mint {
            msg!("Invalid Mint");
            return Err(ProgramError::InvalidAccountData);
        }

        let (vault_seed, bump_seed, size) = if withdraw_mint == escrow_data.pubkey_mint_y {
            (b"vault_y", escrow_data.vault_y_bump, escrow_data.size_y)
        } else if withdraw_mint == escrow_data.pubkey_mint_x {
            (b"vault_x", escrow_data.vault_x_bump, escrow_data.size_x)
        } else {
            msg!("Invalid Mint");
            return Err(ProgramError::InvalidAccountData);
        };

        msg!("Validating vault");
        let vault_seeds = &[
            vault_seed,
            escrow_data.pubkey_alice.as_ref(),
            escrow_data.pubkey_bob.as_ref(),
            escrow_data.pubkey_mint_x.as_ref(),
            escrow_data.pubkey_mint_y.as_ref(),
            pass.as_ref(),
            &[bump_seed],
        ];
        let vault_pubkey = Pubkey::create_program_address(vault_seeds, program_id)?;
        if vault_pubkey != *vault_info.key {
            msg!("Vault key mismatch");
            return Err(ProgramError::InvalidAccountData);
        }
        msg!("Validating escrow data");

        let escrow_seeds = &[
            b"escrow",
            escrow_data.pubkey_alice.as_ref(),
            escrow_data.pubkey_bob.as_ref(),
            escrow_data.pubkey_mint_x.as_ref(),
            escrow_data.pubkey_mint_y.as_ref(),
            pass.as_ref(),
            &[escrow_data.escrow_bump],
        ];

        let escrow_key = Pubkey::create_program_address(escrow_seeds, program_id)?;
        if escrow_key != *escrow_info.key {
            msg!("Escrow key mismatch");
            return Err(ProgramError::InvalidAccountData);
        }
        msg!("Sending transfer");

        solana_program::program::invoke_signed(
            &transfer(
                token_program_info.key,
                &vault_info.key,
                taker_token_info.key,
                &escrow_info.key,
                &[],
                size,
            )?,
            &[
                vault_info.clone(),
                escrow_info.clone(),
                taker_token_info.clone(),
                token_program_info.clone(),
            ],
            &[escrow_seeds],
        )?;

        escrow_data.serialize(&mut *escrow_info.data.borrow_mut())?;

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
    payer_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    pass: [u8; 32],
    vault_seed: &[u8],
) -> Result<u8, ProgramError> {
    let space = Account::LEN;
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
        pass.as_ref(),
    ];

    let (_, bump_seed) = Pubkey::find_program_address(seeds, program_id);
    let seeds = &[
        vault_seed,
        alice_info.key.as_ref(),
        bob_info.key.as_ref(),
        mint_x_info.key.as_ref(),
        mint_y_info.key.as_ref(),
        pass.as_ref(),
        &[bump_seed],
    ];
    msg!("creeat_vault 1");
    msg!("seeds: {:?}", seeds);
    solana_program::program::invoke_signed(
        &system_instruction::create_account(
            payer_info.key,    //from_pubkey
            vault_info.key,    //to_pubkey
            required_lamports, //lamports
            space as u64,      //space
            token_program_info.key,
        ),
        &[
            payer_info.clone(),
            vault_info.clone(),
            system_program_info.clone(),
        ],
        &[seeds],
    )?;
    msg!("create_vault 2");
    Ok(bump_seed)
}
