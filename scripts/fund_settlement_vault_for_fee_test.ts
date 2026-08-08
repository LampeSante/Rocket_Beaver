import {
  Connection,
  Keypair,
  PublicKey,
} from "@solana/web3.js";

import {
  getOrCreateAssociatedTokenAccount,
  mintTo,
  transfer,
} from "@solana/spl-token";

import fs from "fs";

async function main() {
  const connection =
    new Connection(
      "https://api.devnet.solana.com",
      "confirmed",
    );

  const wallet = Keypair.fromSecretKey(
    Uint8Array.from(
      JSON.parse(
        fs.readFileSync(
          process.env.ANCHOR_WALLET!,
          "utf8",
        ),
      ),
    ),
  );

  const mint = new PublicKey(
    "AQBeV6J7GkSJoCstT8dE7Su69o29AHWRAnWMMYW24bA1",
  );

  const treasuryVault = new PublicKey(
    "3xouK9KYZrWPyv8qvsDPwRtKy3rXvvsP4cQs7BfoPCa6",
  );

  const source =
    await getOrCreateAssociatedTokenAccount(
      connection,
      wallet,
      mint,
      wallet.publicKey,
    );

  const amount =
    100_000n * 1_000_000n; // 100k settlement tokens

  const mintSig =
    await mintTo(
      connection,
      wallet,
      mint,
      source.address,
      wallet,
      amount,
    );

  console.log(
    "Minted:",
    mintSig,
  );

  const transferSig =
    await transfer(
      connection,
      wallet,
      source.address,
      treasuryVault,
      wallet,
      amount,
    );

  console.log(
    "Transferred:",
    transferSig,
  );

  console.log(
    JSON.stringify(
      {
        source: source.address.toBase58(),
        treasuryVault:
          treasuryVault.toBase58(),
        amount:
          amount.toString(),
      },
      null,
      2,
    ),
  );
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
