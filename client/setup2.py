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

def process(args):
    op_type = args.op_type
    user = args.user
    if args.payerKey:
        payer_loaded_account = solana.keypair.Keypair.generate(args.payerKey)
    else:
        payer_loaded_account = solana.keypair.Keypair.generate()
    
    if args.aliceKey:
        alice_keypair = solana.keypair.Keypair.generate(args.aliceKey)
    else:
        alice_keypair = solana.keypair.Keypair.generate()

    if args.bobKey:
        bob_keypair = solana.keypair.Keypair.generate(args.bobKey)
    else:
        bob_keypair = solana.keypair.Keypair.generate()

    http_client = Client(args.http)
    payer_public_key = payer_loaded_account.public_key
    alice_pubkey = alice_keypair.public_key
    bob_pubkey = bob_keypair.public_key


    print(f"Payer Account: {payer_public_key}")
    print(f"alice Account: {alice_pubkey}")
    print(f"bob Account: {bob_pubkey}")

    if args.payerKey is None:
        http_client.request_airdrop(payer_loaded_account.public_key, 1000000000)
        time.sleep(15)
    if args.aliceKey is None:
        http_client.request_airdrop(alice_pubkey, 1000000000)
        time.sleep(15)
    if args.bobKey is None:
        http_client.request_airdrop(bob_pubkey, 1000000000)
        time.sleep(15)
   

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

    x_tokens = X_mint_account_address.get_account_info(alice_X_token_account).amount
    y_tokens = Y_mint_account_address.get_account_info(bob_Y_token_account).amount
    if x_tokens < args.numx:
        X_mint_account_address.mint_to(alice_X_token_account,payer_loaded_account,args.xtoken)
    if y_tokens < args.numy:
        Y_mint_account_address.mint_to(bob_Y_token_account,payer_loaded_account,args.ytoken)
    
    # escrow meta-data account
    deployed_program_key = glob.glob("./dist/program/*.json")[0]
    deployed_program_key_account = solana.keypair.Keypair(json.load(open(deployed_program_key))[:32])
    program_id = deployed_program_key_account.public_key
    print(f"Program ID: {program_id}")
    # Seed+password for sending data
    password = _encode('password')
    x_seeds = [
        bytes(args.xpass, encoding='utf8'),
        bytes(alice_pubkey),
        bytes(bob_pubkey),
        bytes(X_mint_account_address.pubkey),
        bytes(Y_mint_account_address.pubkey),
        password,
    ]

    y_seeds = [
        bytes(args.ypass, encoding='utf8'),
        bytes(alice_pubkey),
        bytes(bob_pubkey),
        bytes(X_mint_account_address.pubkey),
        bytes(Y_mint_account_address.pubkey),
        password,
    ]

    escrow_seeds = [
        bytes(args.epass, encoding='utf8'),
        bytes(alice_pubkey),
        bytes(bob_pubkey),
        bytes(X_mint_account_address.pubkey),
        bytes(Y_mint_account_address.pubkey),
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
        tx = Transaction()
        tx_instruction = TransactionInstruction(
            program_id=program_id,
            keys=[
                AccountMeta(pubkey=escrow_address, is_signer=False, is_writable=True),
                AccountMeta(pubkey=X_mint_account_address.pubkey, is_signer=False, is_writable=False),
                AccountMeta(pubkey=Y_mint_account_address.pubkey, is_signer=False, is_writable=False),
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
    elif op_type=='deposit':
        if user=='alice':
            # Alice deposit X
            print('Alice is depositing X')
            data = pack('<B', 1)+password
            tx = Transaction()
            tx_instruction = TransactionInstruction(
                program_id=program_id,
                keys=[
                    AccountMeta(pubkey=escrow_address, is_signer=False, is_writable=True),
                    AccountMeta(pubkey=alice_X_token_account, is_signer=False, is_writable=True), # x_a info
                    AccountMeta(pubkey=vaultx, is_signer=False, is_writable=True),
                    AccountMeta(pubkey=alice_pubkey, is_signer=True, is_writable=False),
                    AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
                    ],
                data=  data,
            )

            tx = tx.add(tx_instruction)
            transaction_results = http_client.send_transaction(tx, *[alice_keypair])

            time.sleep(30)
        elif user=='bob':
            print('Bob is depositing Y')
            # Bob deposit Y
            data = pack('<B', 1)+password
            tx = Transaction()
            tx_instruction = TransactionInstruction(
                program_id=program_id,
                keys=[
                    AccountMeta(pubkey=escrow_address, is_signer=False, is_writable=True),
                    AccountMeta(pubkey=bob_Y_token_account, is_signer=False, is_writable=True),
                    AccountMeta(pubkey=vaulty, is_signer=False, is_writable=True),
                    AccountMeta(pubkey=bob_pubkey, is_signer=True, is_writable=False),
                    AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
                    ],
                data=  data,
            )

            tx = tx.add(tx_instruction)


            transaction_results = http_client.send_transaction(tx, *[bob_keypair])

def get_parser():
    """
    Creates a new argument parser.
    """
    parser = argparse.ArgumentParser(description='Send Escrow Data')

    parser.add_argument('--aliceKey', '-a', help='alice private key', default=None)
    parser.add_argument('--bobKey', '-b', help='bob private key', default=None)
    parser.add_argument('--payerKey', '-p', help='payer private key: must have sols', default=None)
    parser.add_argument('--http', help='http client', default='https://api.devnet.solana.com')
    parser.add_argument('--password', '-X', help='Password', default='passcode')
    parser.add_argument('--numx', help='Min X tokens', default=1000)
    parser.add_argument('--numy', help='Min Y tokens', default=100)
    parser.add_argument('--pid', help='Program ID', required=False)
    parser.add_argument('--xpass', help='Vault X Pass', default="vault_x")
    parser.add_argument('--ypass', help='Vault Y Pass', default="vault_y")
    parser.add_argument('--epass', help='Escrow Pass', default="escrow")
    parser.add_argument('--xtoken', help='Send #X tokens', default=15)
    parser.add_argument('--ytoken', help='Send #Y tokens', default=13)
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