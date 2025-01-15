#![allow(dead_code)]

use std::str::FromStr;

use solana_sdk::{
    instruction::{ AccountMeta, Instruction },
    pubkey::Pubkey,
    signature::Keypair,
    transaction::Transaction,
};

// testing with new idl
#[derive(Debug)]
pub struct Turbin3PrereqProgram;

impl Turbin3PrereqProgram {
    pub const PROGRAM_ID: &'static str = "ADcaide4vBtKuyZQqdU689YqEGZMCmS4tL35bdTv9wJa";

    pub fn get_program_id() -> Pubkey {
        Pubkey::from_str(Self::PROGRAM_ID).unwrap()
    }

    pub fn derive_program_address(seeds: &[&[u8]]) -> Pubkey {
        let (pda, _) = Pubkey::find_program_address(seeds, &Self::get_program_id());
        pda
    }

    pub fn complete(
        accounts: &[&Pubkey],
        args: &CompleteArgs,
        fee_payer: Option<&Pubkey>,
        signers: &[&Keypair],
        recent_blockhash: solana_sdk::hash::Hash
    ) -> Transaction {
        let data = {
            let mut data = vec![0, 77, 224, 147, 136, 25, 88, 76]; // Complete instruction discriminator
            // Serialize github bytes
            let mut github_len = (args.github.len() as u32).to_le_bytes().to_vec();
            data.append(&mut github_len);
            data.extend_from_slice(&args.github);
            data
        };

        let instruction = Instruction {
            program_id: Self::get_program_id(),
            accounts: vec![
                AccountMeta::new(*accounts[0], true), // signer
                AccountMeta::new(*accounts[1], false), // prereq
                AccountMeta::new_readonly(*accounts[2], false) // system_program
            ],
            data,
        };

        Transaction::new_signed_with_payer(&[instruction], fee_payer, signers, recent_blockhash)
    }

    pub fn update(
        accounts: &[&Pubkey],
        args: &UpdateArgs,
        fee_payer: Option<&Pubkey>,
        signers: &[&Keypair],
        recent_blockhash: solana_sdk::hash::Hash
    ) -> Transaction {
        let data = {
            let mut data = vec![219, 200, 88, 176, 158, 63, 253, 127]; // Update instruction discriminator
            // Serialize github bytes
            let mut github_len = (args.github.len() as u32).to_le_bytes().to_vec();
            data.append(&mut github_len);
            data.extend_from_slice(&args.github);
            data
        };

        let instruction = Instruction {
            program_id: Self::get_program_id(),
            accounts: vec![
                AccountMeta::new(*accounts[0], true), // signer
                AccountMeta::new(*accounts[1], false), // prereq
                AccountMeta::new_readonly(*accounts[2], false) // system_program
            ],
            data,
        };

        Transaction::new_signed_with_payer(&[instruction], fee_payer, signers, recent_blockhash)
    }
}

pub struct CompleteArgs {
    pub github: Vec<u8>,
}

pub struct UpdateArgs {
    pub github: Vec<u8>,
}

// use solana_idlgen::idlgen;
// idlgen!({
//     "version": "0.1.0",
//     "name": "Turbin3_prereq",
//     "instructions": [
//       {
//         "name": "complete",
//         "accounts": [
//           {
//             "name": "signer",
//             "isMut": true,
//             "isSigner": true
//           },
//           {
//             "name": "prereq",
//             "isMut": true,
//             "isSigner": false
//           },
//           {
//             "name": "systemProgram",
//             "isMut": false,
//             "isSigner": false
//           }
//         ],
//         "args": [
//           {
//             "name": "github",
//             "type": "bytes"
//           }
//         ]
//       },
//       {
//         "name": "update",
//         "accounts": [
//           {
//             "name": "signer",
//             "isMut": true,
//             "isSigner": true
//           },
//           {
//             "name": "prereq",
//             "isMut": true,
//             "isSigner": false
//           },
//           {
//             "name": "systemProgram",
//             "isMut": false,
//             "isSigner": false
//           }
//         ],
//         "args": [
//           {
//             "name": "github",
//             "type": "bytes"
//           }
//         ]
//       }
//     ],
//     "accounts": [
//       {
//         "name": "PrereqAccount",
//         "type": {
//           "kind": "struct",
//           "fields": [
//             {
//               "name": "github",
//               "type": "bytes"
//             },
//             {
//               "name": "key",
//               "type": "publicKey"
//             }
//           ]
//         }
//       }
//     ],
//     "errors": [
//       {
//         "code": 6000,
//         "name": "InvalidGithubAccount",
//         "msg": "Invalid Github account"
//       }
//     ],
//     "metadata": {
//         "address": "HC2oqz2p6DEWfrahenqdq2moUcga9c9biqRBcdK3XKU1"
//          }
//   });
