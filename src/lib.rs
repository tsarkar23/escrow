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
};

// use borsh::{BorshDeserialize, BorshSerialize};
// use spl_token;
// use solana_program::{
//     borsh::{BorshDeserialize, BorshSerialize},
//     system_instruction,
//     account_info::{next_account_info, AccountInfo},
//     entrypoint,
//     entrypoint::ProgramResult,
//     msg,
//     program_error::ProgramError,
//     pubkey::Pubkey,
//     sysvar::{rent::Rent, Sysvar},
// };

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

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // Alice pubkey, Bob pubkey, X mint pub key, Y mint pub key, Vault X, Va
    _instruction_data: &[u8], // pubkey, school, email, and link
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

    msg!("Step 4");

    // let (escrow_pubkey2, escrow_bump_seed) = Pubkey::find_program_address(escrow_seeds, &program_id);
    // msg!("Step 5");
    // let (vaultx_pubkey, vaultx_bump_seed) = Pubkey::find_program_address(vault_x_seeds, &program_id);
    // msg!("Step 6");
    // let (vaulty_pubkey, valuty_bump_seed) = Pubkey::find_program_address(vault_y_seeds, &program_id);

    msg!("Step 7");


    // pub struct AccountInfo<'a> {
    //     pub key: &'a Pubkey,
    //     pub is_signer: bool,
    //     pub is_writable: bool,
    //     pub lamports: Rc<RefCell<&'a mut u64>>,
    //     pub data: Rc<RefCell<&'a mut [u8]>>,
    //     pub owner: &'a Pubkey,
    //     pub executable: bool,
    //     pub rent_epoch: u64,
    // }


    // let vault_x_address = 
    create_vault_account(program_id, &[
        vault_x_info.clone(),
        mint_x_info.clone(),
        escrow_info.clone(), // account owner info
        payer_info.clone(),
        token_program_info.clone(),
        rent_info.clone(),
        system_program_info.clone(),],
        &[b"alice", &[251u8]]       
    )?;

    msg!("Step 8");

    // let vault_y_address = 
    create_vault_account(program_id, &[
        vault_y_info.clone(),
        mint_y_info.clone(),
        escrow_info.clone(), // account owner info
        payer_info.clone(),
        token_program_info.clone(),
        rent_info.clone(),
        system_program_info.clone(),],
        &[b"bob", &[255u8]]
    )?;

    msg!("The vaults are created!");

    // // Iterating accounts is safer then indexing
    // let accounts_iter = &mut accounts.iter();

    // // Get the account to say hello to
    // let account = next_account_info(accounts_iter)?;

    // // The account must be owned by the program in order to modify its data
    // if account.owner != program_id {
    //     msg!("Greeting account owner {} is not equal to the program id{}", &program_id, &account.owner);
    //     // msg!("program_id is: {}", &program_id);
    //     // msg!("account.owner is: {}", &account.owner);
    //     return Err(ProgramError::IncorrectProgramId);
    // }

    // let args = Args:try_from_slice(_instruction_data)?;
    // args.arg0

    // just put args[1] in the account.data
    let mut args_account = Amounts::try_from_slice(_instruction_data)?; 
    let info = Info{xval:args_account.xval,yval:args_account.yval,state:1};
    info.serialize(&mut &mut escrow_info.data.borrow_mut()[..])?;

    // msg!("Public key: {}, school: {}, email:{}, and the link: {}", args_account.pubkey, args_account.school, args_account.email, args_account.link);

    Ok(())
}



fn create_vault_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    seeds: &[&[u8]],
) -> ProgramResult {
    msg!("Step 9");
    let account_info_iter = &mut accounts.iter(); 
    let vault_info = next_account_info(account_info_iter)?; // ? is it the key pair of vault or what?
    let mint_info = next_account_info(account_info_iter)?; // mind public address
    let account_owner_info = next_account_info(account_info_iter)?; // pda?
    let payer_info = next_account_info(account_info_iter)?; // payer_account, is it both public and private key?
    let token_program_info = next_account_info(account_info_iter)?; // token_program_id
    let rent_info = next_account_info(account_info_iter)?; // solana.py from solana.sysvar import SYSVAR_RENT_PUBKEY
    let system_program_info = next_account_info(account_info_iter)?; // system program public key? public_key(1)?
    // let rent: Rent = bincode::deserialize(&rent_info.data.borrow()).map_err(|_| ProgramError::InvalidArgument)?;
    let rent = &Rent::from_account_info(rent_info)?;
    let space: u64 =  165; // spl_token::state::Account::LEN;
    let required_lamports = rent
        .minimum_balance(space.try_into().unwrap())
        .max(1)
        .saturating_sub(vault_info.lamports());
    msg!("Step 10");
    solana_program::program::invoke_signed(
        &system_instruction::create_account(
            payer_info.key, //from_pubkey
            vault_info.key, //to_pubkey
            required_lamports, //lamports
            space.try_into().unwrap(), //space
            //program_id, //owner
        token_program_info.key,
        ),
        &[payer_info.clone(), vault_info.clone(), system_program_info.clone()],
        &[seeds],
    )?;
    msg!("Step 11");
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
        &[seeds]
    )?;
Ok(())
}

// Sanity tests
#[cfg(test)]
mod test {
    use super::*;
    use solana_program::clock::Epoch;
    use std::mem;

    #[test]
    fn test_sanity() {
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let mut lamports = 0;
        let mut data = vec![0; mem::size_of::<u32>()];
        let owner = Pubkey::default();
        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            Epoch::default(),
        );
        let instruction_data: Vec<u8> = Vec::new();

        let accounts = vec![account];

        assert_eq!(
            GreetingAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            0
        );
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        assert_eq!(
            GreetingAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            1
        );
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        assert_eq!(
            GreetingAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            2
        );
    }
}



// /// Generate a derived program address
// ///     * seeds, symbolic keywords used to derive the key
// ///     * program_id, program that the address is derived for
// pub fn create_program_address(
//     seeds: &[&[u8]],
//     program_id: &Pubkey,
// ) -> Result<Pubkey, PubkeyError>

// /// Find a valid off-curve derived program address and its bump seed
// ///     * seeds, symbolic keywords used to derive the key
// ///     * program_id, program that the address is derived for
// pub fn find_program_address(
//     seeds: &[&[u8]],
//     program_id: &Pubkey,
// ) -> Option<(Pubkey, u8)> {
//     let mut bump_seed = [std::u8::MAX];
//     for _ in 0..std::u8::MAX {
//         let mut seeds_with_bump = seeds.to_vec();
//         seeds_with_bump.push(&bump_seed);
//         if let Ok(address) = create_program_address(&seeds_with_bump, program_id) {
//             return Some((address, bump_seed[0]));
//         }
//         bump_seed[0] -= 1;
//     }
//     None
// }