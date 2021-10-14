# accounts
import json
import solana
import time
import argparse
import sys
import glob
from solana.rpc.api import Client
from solana.blockhash import Blockhash
from solana.publickey import PublicKey
from solana.system_program import CreateAccountWithSeedParams, create_account_with_seed, CreateAccountParams, create_account
from solana.transaction import Transaction, AccountMeta, TransactionInstruction
from solana.system_program import SYS_PROGRAM_ID
from spl.token.client import Token
from solana import keypair
from solana.publickey import PublicKey
from solana.sysvar import SYSVAR_RENT_PUBKEY
from struct import *
import struct

def _encode(s, pad=32):
    r = s.encode("UTF-8")
    r += bytes([0] * (pad - len(r)))
    return r

TOKEN_PROGRAM_ID: PublicKey = PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")

def process_init(program_id, escrow_address, x_mint_pubkey,\
                 y_mint_pubkey, vaultx, vaulty, payer_public_key,\
                 alice_pubkey, bob_pubkey, data, http_client, payer_loaded_account):
    tx = Transaction()
    tx_instruction = TransactionInstruction(
        program_id=program_id,
        keys=[
            AccountMeta(pubkey=escrow_address, is_signer=False, is_writable=True),
            AccountMeta(pubkey=x_mint_pubkey, is_signer=False, is_writable=False),
            AccountMeta(pubkey=y_mint_pubkey, is_signer=False, is_writable=False),
            AccountMeta(pubkey=vaultx, is_signer=False, is_writable=True),
            AccountMeta(pubkey=vaulty, is_signer=False, is_writable=True),
            AccountMeta(pubkey=payer_public_key, is_signer=True, is_writable=False),
            AccountMeta(pubkey=alice_pubkey, is_signer=False, is_writable=False),
            AccountMeta(pubkey=bob_pubkey, is_signer=False, is_writable=False),
            AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(pubkey=SYSVAR_RENT_PUBKEY, is_signer=False, is_writable=False),
            AccountMeta(pubkey=SYS_PROGRAM_ID, is_signer=False, is_writable=False),
            ],
        data=data,
    )
    tx = tx.add(tx_instruction)
    transaction_results = http_client.send_transaction(tx, *[payer_loaded_account])#, *[payer_loaded_account])
    print('Initializing Accounts Ended') 
    time.sleep(30)   

def process_deposit(program_id, escrow_address, user_token_account,\
                    vault, user_pubkey, data, http_client,\
                    user_keypair):
    tx = Transaction()
    tx_instruction = TransactionInstruction(
        program_id=program_id,
        keys=[
            AccountMeta(pubkey=escrow_address, is_signer=False, is_writable=True),
            AccountMeta(pubkey=user_token_account, is_signer=False, is_writable=True), # x_a info
            AccountMeta(pubkey=vault, is_signer=False, is_writable=True),
            AccountMeta(pubkey=user_pubkey, is_signer=True, is_writable=False),
            AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            ],
        data=  data,
    )

    tx = tx.add(tx_instruction)
    transaction_results = http_client.send_transaction(tx, *[user_keypair])

    time.sleep(30)  


def process(args):
    op_type = args.op_type
    user = args.user
    http_client = Client(args.http)

    # Payer details
    if args.payerKey:
        payer_loaded_account = solana.keypair.Keypair(json.load(open(args.payerKey))[:32])
    elif op_type=='init':
        print('Need a payer; generating new')
        payer_loaded_account = solana.keypair.Keypair.generate()
        tmp = [int(x) for x in payer_loaded_account.secret_key]
        with open('./payer_secret_key.txt', 'w') as w:
            json.dump(tmp, w)
        print('Air drop solana for init; not on mainnet')
        http_client.request_airdrop(payer_loaded_account.public_key, 1000000000)
        time.sleep(15)
    else:
        payer_loaded_account = None
        payer_public_key = None
        print('Payer details not needed')

    # Alice details    
    if args.aliceKey:
        alice_keypair = solana.keypair.Keypair(json.load(open(args.aliceKey))[:32])
    elif op_type=='init':
        print('Need Alice; generating new')
        alice_keypair = solana.keypair.Keypair.generate()
        tmp = [int(x) for x in alice_keypair.secret_key]
        with open('./alice_secret_key.txt', 'w') as w:
            json.dump(tmp, w)
        with open('./alice_public_key.txt', 'w') as w:
            json.dump(str(alice_keypair.public_key), w)
        print('Air drop solana for init; not on mainnet')
        http_client.request_airdrop(alice_keypair.public_key, 1000000000)
        time.sleep(15)
    elif user=='alice':
        print('Alice details needed')
        return
    else:
        print('Nothing to do for Alice')
        alice_keypair = None
        if args.apubk is None:
            print('Need Alice public key!!!')
            return
        alice_pubkey = PublicKey(json.load(open(args.apubk)))

    # Bob details
    if args.bobKey: 
        bob_keypair = solana.keypair.Keypair(json.load(open(args.bobKey))[:32])
    elif op_type=='init':
        print('Need Bob; generating new')
        bob_keypair = solana.keypair.Keypair.generate()
        tmp = [int(x) for x in bob_keypair.secret_key]
        with open('./bob_secret_key.txt', 'w') as w:
            json.dump(tmp, w)
        with open('./bob_public_key.txt', 'w') as w:
            json.dump(str(bob_keypair.public_key), w)
        print('Air drop solana for init; not on mainnet')
        http_client.request_airdrop(bob_keypair.public_key, 1000000000)
        time.sleep(15)
    elif user=='bob':
        print('Bob details needed')
        return
    else:
        print('Nothing to do for Bob')
        bob_keypair = None
        if args.bpubk is None:
            print('Need Bob public key!!!')
            return
        bob_pubkey = PublicKey(json.load(open(args.bpubk)))
    
    # Program details
    program_id = PublicKey(args.pid)

    if alice_keypair is not None:
        alice_pubkey = alice_keypair.public_key
    if bob_keypair is not None:
        bob_pubkey = bob_keypair.public_key
    if payer_loaded_account is not None:
        payer_public_key = payer_loaded_account.public_key

    print(f"payer Account: {payer_public_key}")
    print(f"alice Account: {alice_pubkey}")
    print(f"bob Account: {bob_pubkey}")
   

    # Always create token accounts
    if op_type=='init':
        X_mint_account_address = Token.create_mint(conn= http_client, payer = payer_loaded_account,\
                                                mint_authority = payer_loaded_account.public_key,\
                                                decimals = 0, program_id = TOKEN_PROGRAM_ID, )
        Y_mint_account_address = Token.create_mint(conn= http_client, payer = payer_loaded_account,\
                                                mint_authority = payer_loaded_account.public_key,\
                                                decimals = 0, program_id = TOKEN_PROGRAM_ID, )
        alice_X_token_account = X_mint_account_address.create_associated_token_account(alice_pubkey)
        alice_Y_token_account = Y_mint_account_address.create_associated_token_account(alice_pubkey)
        bob_X_token_account = X_mint_account_address.create_associated_token_account(bob_pubkey)
        bob_Y_token_account = Y_mint_account_address.create_associated_token_account(bob_pubkey)

        X_mint_account_address.mint_to(alice_X_token_account,payer_loaded_account,args.xtoken)
        Y_mint_account_address.mint_to(bob_Y_token_account,payer_loaded_account,args.ytoken)

        # Writing all mint details
        with open('./mint_x.txt', 'w') as w:
            json.dump(str(X_mint_account_address.pubkey), w)
        with open('./mint_y.txt', 'w') as w:
            json.dump(str(Y_mint_account_address.pubkey), w)
        with open('./alice_x_token.txt', 'w') as w:
            json.dump(str(alice_X_token_account), w)
        with open('./alice_y_token.txt', 'w') as w:
            json.dump(str(alice_Y_token_account), w)        
        with open('./bob_x_token.txt', 'w') as w:
            json.dump(str(bob_X_token_account), w)
        with open('./bob_y_token.txt', 'w') as w:
            json.dump(str(bob_Y_token_account), w)   
           
        x_mint_pubkey = X_mint_account_address.pubkey
        y_mint_pubkey = Y_mint_account_address.pubkey

    elif user=='alice':
        x_mint_pubkey = PublicKey(json.load(open(args.mintx)))
        y_mint_pubkey = PublicKey(json.load(open(args.minty)))
        alice_X_token_account = PublicKey(json.load(open(args.aTokenX)))        

    elif user=='bob':
        x_mint_pubkey = PublicKey(json.load(open(args.mintx)))
        y_mint_pubkey = PublicKey(json.load(open(args.minty)))
        bob_Y_token_account = PublicKey(json.load(open(args.bTokenY)))

    # Seed+password for sending data
    password = _encode(str(args.password))

    x_seeds = [
        bytes(str(args.xpass), encoding='utf8'),
        bytes(alice_pubkey),
        bytes(bob_pubkey),
        bytes(x_mint_pubkey),
        bytes(y_mint_pubkey),
        password,
    ]

    y_seeds = [
        bytes(str(args.ypass), encoding='utf8'),
        bytes(alice_pubkey),
        bytes(bob_pubkey),
        bytes(x_mint_pubkey),
        bytes(y_mint_pubkey),
        password,
    ]

    escrow_seeds = [
        bytes(str(args.epass), encoding='utf8'),
        bytes(alice_pubkey),
        bytes(bob_pubkey),
        bytes(x_mint_pubkey),
        bytes(y_mint_pubkey),
        password,
    ]

    # create vaults, necessary?
    vaultx, xseed = PublicKey.find_program_address(seeds=x_seeds,program_id=program_id)
    vaulty, yseed = PublicKey.find_program_address(seeds=y_seeds,program_id=program_id)
    escrow_address, escrow_seed = PublicKey.find_program_address(seeds=escrow_seeds,program_id=program_id)


    if op_type == 'init': 
        print('Start Initializing Accounts')
        # initialize transaction
        x_val = args.xtoken
        y_val = args.ytoken
        data = pack('<BQQ', 0,x_val,y_val)+password
        process_init(program_id, escrow_address, x_mint_pubkey,\
                     y_mint_pubkey, vaultx, vaulty, payer_public_key,\
                     alice_pubkey, bob_pubkey, data, http_client, payer_loaded_account)
    
    elif op_type=='deposit':
        if user=='alice':
            
            # Alice deposit X
            print('Alice is depositing X')
            data = pack('<B', 1)+password
            process_deposit(program_id, escrow_address, alice_X_token_account, vaultx,\
                            alice_pubkey, data, http_client, alice_keypair)
        elif user=='bob':
            print('Bob is depositing Y')
            # Bob deposit Y
            data = pack('<B', 1)+password
            process_deposit(program_id, escrow_address, bob_Y_token_account, vaulty,\
                            bob_pubkey, data, http_client, bob_keypair)

def get_parser():
    """
    Creates a new argument parser.
    """
    parser = argparse.ArgumentParser(description='Send Escrow Data')

    parser.add_argument('--aliceKey', '-a', help='alice private key file', default=None)
    parser.add_argument('--bobKey', '-b', help='bob private key file', default=None)
    parser.add_argument('--apubk', help='alice public key file', default=None)
    parser.add_argument('--bpubk', help='bob public key file', default=None)
    parser.add_argument('--payerKey', '-p', help='payer private key: must have sols', default=None)
    parser.add_argument('--http', help='http client', default='https://api.devnet.solana.com')
    parser.add_argument('--password', '-X', help='Password', default='password')
    parser.add_argument('--numx', help='Min X tokens', default=1000)
    parser.add_argument('--numy', help='Min Y tokens', default=100)
    parser.add_argument('--pid', help='Program ID', required=True)
    parser.add_argument('--xpass', help='Vault X Pass', default="vault_x")
    parser.add_argument('--ypass', help='Vault Y Pass', default="vault_y")
    parser.add_argument('--epass', help='Escrow Pass', default="escrow")
    parser.add_argument('--xtoken', help='Send #X tokens', default=15)
    parser.add_argument('--ytoken', help='Send #Y tokens', default=13)
    parser.add_argument('--aTokenX', help='Alice X Token Account File', default=None)
    parser.add_argument('--bTokenY', help='Bob Y Token Account File', default=None)
    parser.add_argument('--mintx', help='X Mint Account Public Key File', default=None)
    parser.add_argument('--minty', help='Y Mint Account Public Key File', default=None)
    parser.add_argument('--op_type', help='Operation Type', default='init', choices=['init', 'deposit', 'withdraw'])
    parser.add_argument('--user', help='Alice/Bob', default='alice', choices=['alice', 'bob'])

    return parser


def main(args=None):
    """
    Main entry point for your project.
    Args:
        args : list
            A of arguments as if they were input in the command line. Leave it
            None to use sys.argv.
    """

    parser = get_parser()
    args = parser.parse_args(args)

    process(args)


if __name__ == '__main__':
    main()