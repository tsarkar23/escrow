#!/bin/sh

cargo build-bpf && solana program deploy /Users/MohammadHossein/Workspaces/rust/escrow/target/deploy/escrow.so
