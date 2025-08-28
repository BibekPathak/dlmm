import { AnchorProvider, setProvider, Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";

describe("dlmm", () => {
  const provider = AnchorProvider.env();
  setProvider(provider);

  it("boots provider", async () => {
    const walletPk = provider.wallet.publicKey;
    console.log("Provider pubkey:", walletPk.toBase58());
  });
});


