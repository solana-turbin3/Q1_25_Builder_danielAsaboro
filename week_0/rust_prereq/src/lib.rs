use std::{ fs::File, io::Write };

use solana_sdk::signature::Keypair;

mod programs;

pub fn save_keypair(filepath: &str, kp: &Keypair) -> std::io::Result<()> {
    let bytes = kp.to_bytes();
    // Save as JSON array
    let json = serde_json::to_string_pretty(&bytes.to_vec())?;

    let mut file = File::create(filepath)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use solana_program::system_program;
    use solana_sdk::signature::{ Keypair, Signer, read_keypair_file };
    use bs58;
    use std::{ io::{ self, BufRead }, path::Path };

    use solana_client::rpc_client::RpcClient;
    use solana_program::{ pubkey::Pubkey, system_instruction::transfer };

    use solana_sdk::transaction::Transaction;
    use std::str::FromStr;
    use solana_sdk::message::Message;

    use crate::{
        programs::turbin3_prereq::{ CompleteArgs, Turbin3PrereqProgram, UpdateArgs },
        save_keypair,
    };

    const RPC_URL: &str = "https://api.devnet.solana.com";
    #[test]
    fn keygen() {
        let filepath = "dev-wallet.json";
        let path = Path::new(filepath);

        if path.exists() {
            println!("Wallet file already exists at: {}", filepath);
            println!("Would you like to override it? (y/N):");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("can't parse the inputted value");

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("File at path `{}` isn't empty; Choose another file path", filepath);
                panic!("User cancelled save operation");
            }
        }
        // Create a new keypair
        let kp = Keypair::new();

        // Save the keypair as JSON array
        save_keypair(filepath, &kp).expect("Failed to save keypair");

        println!("You've generated a new Solana wallet: {}", kp.pubkey().to_string());
        println!("Saved to: {}", filepath);
        println!("");
        println!("To save your wallet somewhere, copy and paste the following into a JSON file:");
        println!("{:?}", kp.to_bytes());
    }

    #[test]
    fn airdrop() {
        // Import our keypair
        let keypair = read_keypair_file("dev-wallet5.json").expect("Couldn't find wallet file");

        // Connected to Solana Devnet RPC Client
        let client = RpcClient::new(RPC_URL);

        // We're going to claim 2 devnet SOL tokens (2 billion lamports)
        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(s) => {
                println!("Success! Check out your TX here:");
                println!("https://explorer.solana.com/tx/{}?cluster=devnet", s.to_string());
            }
            Err(e) => println!("Oops, something went wrong: {}", e.to_string()),
        };
    }

    #[test]
    fn check_public_key() {
        let keypair = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");

        print!(" the private key is thus {} ", keypair.pubkey());
    }

    #[test]
    fn transfer_point_one_sol() {
        // Import our keypair
        let sender_keypair = read_keypair_file("dev-wallet.json").expect(
            "Couldn't find wallet file"
        );

        // it's rare you have the reciever whole keypair details,
        //
        // let to_pubkey = read_keypair_file("./Turbin3-wallet.json")
        //     .expect("Couldn't find receiver wallet file")
        //     .pubkey();

        let to_pubkey = Pubkey::from_str("95HRCXSxU18oh8hWXbdkqxenqCwopDGXjmhFQ9Pd2EuQ").unwrap();

        // Define our Turbin3 public key
        // let to_pubkey = Pubkey::from_str("95HRCXSxU18oh8hWXbdkqxenqCwopDGXjmhFQ9Pd2EuQ").unwrap();
        // Create a Solana devnet connection
        let rpc_client = RpcClient::new(RPC_URL);

        // Get recent blockhash
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        // Get balance of dev wallet
        let balance = rpc_client
            .get_balance(&sender_keypair.pubkey())
            .expect("Failed to get balance");

        let amount_to_send = 100_000_000u64;

        // Test message to calculate fees
        let test_message = Message::new_with_blockhash(
            &[transfer(&sender_keypair.pubkey(), &to_pubkey, amount_to_send)],
            Some(&sender_keypair.pubkey()),
            &recent_blockhash
        );
        // Calculate transaction fee
        let fee = rpc_client
            .get_fee_for_message(&test_message)
            .expect("Failed to get fee calculator");

        // Check if we have enough balance for transfer + fee
        if balance < amount_to_send + fee {
            panic!("Insufficient balance for transfer + fee");
        }

        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&sender_keypair.pubkey(), &to_pubkey, amount_to_send)],
            Some(&sender_keypair.pubkey()),
            &vec![&sender_keypair],
            recent_blockhash
        );

        // Send the transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        // Print our transaction out
        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
    }

    #[test]
    fn transfer_whole_balance() {
        //
        // Import our keypair
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        // Define our Turbin3 public key
        let to_pubkey = Pubkey::from_str("95HRCXSxU18oh8hWXbdkqxenqCwopDGXjmhFQ9Pd2EuQ").unwrap();
        // Create a Solana devnet connection
        let rpc_client = RpcClient::new(RPC_URL);

        // Get recent blockhash
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        // Get balance of dev wallet
        let balance = rpc_client.get_balance(&keypair.pubkey()).expect("Failed to get balance");

        // Create a test transaction to calculate fees
        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash
        );

        // Calculate exact fee rate to transfer entire SOL amount out of account minus fees
        let fee = rpc_client.get_fee_for_message(&message).expect("Failed to get fee calculator");

        // Check if we have enough balance for transfer + fee
        if balance < fee {
            panic!("Insufficient balance for transfer + fee");
        }

        // Deduct fee from lamports amount and create a TX with correct balance
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash
        );

        // Send the transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        // Print our transaction out
        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
    }

    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as base58:");
        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();
        println!("Your wallet file is:");
        let wallet = bs58::decode(base58).into_vec().unwrap();
        println!("{:?}", wallet);
    }

    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a wallet file byte array:");
        let stdin = io::stdin();
        let wallet = stdin
            .lock()
            .lines()
            .next()
            .unwrap()
            .unwrap()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();

        println!("Your private key is:");
        let base58 = bs58::encode(wallet).into_string();
        println!("{:?}", base58);
    }

    #[test]
    fn enroll_for_rust_prereq() {
        //
        // Create a Solana devnet connection
        let rpc_client = RpcClient::new(RPC_URL);
        // Let's define our accounts
        let signer = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");

        let prereq = Turbin3PrereqProgram::derive_program_address(
            &[b"prereq", signer.pubkey().to_bytes().as_ref()]
        );

        // Define our instruction data
        let args = CompleteArgs {
            github: b"danielAsaboro".to_vec(),
        };

        // Get recent blockhash
        let blockhash = rpc_client.get_latest_blockhash().expect("Failed to get recent blockhash");

        // Now we can invoke the "complete" function
        let transaction = Turbin3PrereqProgram::complete(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],

            blockhash
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        // Print our transaction out
        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
    }

    // added update for completeness sake;
    #[test]
    fn update_enroll_data_for_rust_prereq() {
        //
        // Create a Solana devnet connection
        let rpc_client = RpcClient::new(RPC_URL);
        // Let's define our accounts
        let signer = read_keypair_file("./src/Turbin3-wallet.json").expect(
            "Couldn't find wallet file"
        );

        let prereq = Turbin3PrereqProgram::derive_program_address(
            &[b"prereq", signer.pubkey().to_bytes().as_ref()]
        );

        // Define our instruction data
        let args = UpdateArgs {
            github: b"danielAsaboro".to_vec(),
        };

        // Get recent blockhash
        let blockhash = rpc_client.get_latest_blockhash().expect("Failed to get recent blockhash");

        // Now we can invoke the "complete" function
        let transaction = Turbin3PrereqProgram::update(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],

            blockhash
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        // Print our transaction out
        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
    }

    #[test]
    fn enroll_with_new_idl() {
        // Create a Solana devnet connection
        let rpc_client = RpcClient::new(RPC_URL);

        // Define our accounts
        let signer = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");

        // Derive the PDA for prereq account
        let prereq = Turbin3PrereqProgram::derive_program_address(
            &[b"prereq", signer.pubkey().to_bytes().as_ref()]
        );

        // Define instruction args
        let args = CompleteArgs {
            github: b"danielAsaboro".to_vec(),
        };

        // Get recent blockhash
        let blockhash = rpc_client.get_latest_blockhash().expect("Failed to get recent blockhash");

        // Create and send transaction
        let transaction = Turbin3PrereqProgram::complete(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],
            blockhash
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
    }

    #[test]
    fn update_enroll_details_with_new_idl() {
        // Create a Solana devnet connection
        let rpc_client = RpcClient::new(RPC_URL);

        // Define our accounts
        let signer = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");

        // Derive the PDA for prereq account
        let prereq = Turbin3PrereqProgram::derive_program_address(
            &[b"prereq", signer.pubkey().to_bytes().as_ref()]
        );

        // Define instruction args
        let args = UpdateArgs {
            github: b"danielAsaboro".to_vec(),
        };

        // Get recent blockhash
        let blockhash = rpc_client.get_latest_blockhash().expect("Failed to get recent blockhash");

        // Create and send transaction
        let transaction = Turbin3PrereqProgram::update(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],
            blockhash
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
    }
}
