import {
  Connection,
  PublicKey,
} from "@solana/web3.js";
import {
  getMint,
} from "@solana/spl-token";

async function main() {
  const connection =
    new Connection(
      "https://api.devnet.solana.com",
      "confirmed",
    );

  const mint = new PublicKey(
    "AQBeV6J7GkSJoCstT8dE7Su69o29AHWRAnWMMYW24bA1",
  );

  const info = await getMint(
    connection,
    mint,
  );

  console.log(
    JSON.stringify(
      {
        mint: mint.toBase58(),
        decimals: info.decimals,
        supply: info.supply.toString(),
        mintAuthority:
          info.mintAuthority?.toBase58() ?? null,
      },
      null,
      2,
    ),
  );
}

main().catch(console.error);
