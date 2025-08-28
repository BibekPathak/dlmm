import { PublicKey } from "@solana/web3.js";
import { Program, AnchorProvider, Idl } from "@coral-xyz/anchor";

export class DlmmClient {
  readonly program: Program;

  constructor(readonly provider: AnchorProvider, readonly id: PublicKey) {
    // At this stage, we don't have an IDL, so treat as any
    // Users can replace with generated IDL later
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    this.program = new Program({} as Idl, id, provider as any);
  }
}


