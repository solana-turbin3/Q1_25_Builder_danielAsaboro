import * as bs58 from "bs58";
import { Keypair } from "@solana/web3.js";
import { privateKey } from "./private_key";

export const base58ToKeypair = (base58PrivateKey: string): Keypair => {
  return Keypair.fromSecretKey(bs58.default.decode(base58PrivateKey));
};

const keypair = base58ToKeypair(privateKey);

// Log public key in base58 format (this is your wallet address)
console.log("Public Key:", keypair.publicKey.toBase58());

// Log the full keypair secret key bytes
console.log("Secret Key:", keypair.secretKey);

// If you need the whole keypair object
console.log("Full Keypair:", keypair);
