import { PublicKey } from "@solana/web3.js";
import { Program, AnchorProvider, Idl } from "@coral-xyz/anchor";

import {
  PoolAccount,
  BinArrayAccount,
  PositionAccount,
  InitializePoolParams,
  OpenPositionParams,
  BinDeposit,
  SwapParams,
  SwapQuote,
  BINS_PER_ARRAY,
} from "./types";
import {
  derivePoolAddress,
  deriveBinArrayAddress,
  derivePositionAddress,
  deriveAllBinArrayAddresses,
} from "./pda";
import { quoteSwap, QuoteParams } from "./quote";

export class DlmmClient {
  readonly program: Program;

  constructor(readonly provider: AnchorProvider, readonly id: PublicKey, idl?: Idl) {
    const resolvedIdl = idl || ({ metadata: { address: id.toBase58() } } as never);
    this.program = new Program(resolvedIdl, provider);
  }

  // ── Pool ──────────────────────────────────────────────────────────

  async createPool(
    authority: PublicKey,
    mintA: PublicKey,
    mintB: PublicKey,
    params: InitializePoolParams,
  ): Promise<string> {
    const [poolPda] = derivePoolAddress(this.id, mintA, mintB);
    const tx = await this.program.methods
      .initializePool({
        feeTierBps: params.feeTierBps,
        protocolFeeBps: params.protocolFeeBps,
        binStepBps: params.binStepBps,
        baseBinId: params.baseBinId,
        activeBinId: params.activeBinId,
        baseFeeBps: params.baseFeeBps,
        feeDecayInterval: params.feeDecayInterval,
      } as never)
      .accounts({
        payer: this.provider.wallet.publicKey,
        authority,
        pool: poolPda,
        tokenMintA: mintA,
        tokenMintB: mintB,
        tokenVaultA: poolPda,
        tokenVaultB: poolPda,
        systemProgram: PublicKey.default,
        tokenProgram: PublicKey.default,
        rent: PublicKey.default,
      } as never)
      .rpc();
    return tx;
  }

  async getPool(poolAddress: PublicKey): Promise<PoolAccount | null> {
    try {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const acc = await (this.program.account as any).pool.fetch(poolAddress);
      return acc as PoolAccount;
    } catch {
      return null;
    }
  }

  // ── BinArray ──────────────────────────────────────────────────────

  async getBinArray(address: PublicKey): Promise<BinArrayAccount | null> {
    try {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const acc = await (this.program.account as any).binArray.fetch(address);
      return acc as BinArrayAccount;
    } catch {
      return null;
    }
  }

  async getBinArrays(pool: PublicKey, lowerBinId?: number, upperBinId?: number): Promise<BinArrayAccount[]> {
    const binArrays: BinArrayAccount[] = [];
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const programBinArrays = await (this.program.account as any).binArray.all([
      { memcmp: { offset: 8, bytes: pool.toBase58() } },
    ]);
    for (const ba of programBinArrays) {
      const acc = ba.account as BinArrayAccount;
      if (lowerBinId !== undefined && upperBinId !== undefined) {
        const arrEnd = acc.startBinId + BINS_PER_ARRAY - 1;
        if (acc.startBinId > upperBinId || arrEnd < lowerBinId) continue;
      }
      binArrays.push(acc);
    }
    binArrays.sort((a, b) => a.startBinId - b.startBinId);
    return binArrays;
  }

  async createBinArray(pool: PublicKey, startBinId: number): Promise<string> {
    const [baPda] = deriveBinArrayAddress(this.id, pool, startBinId);
    const tx = await this.program.methods
      .initializeBinArray(startBinId)
      .accounts({
        payer: this.provider.wallet.publicKey,
        pool,
        binArray: baPda,
        systemProgram: PublicKey.default,
      } as never)
      .rpc();
    return tx;
  }

  // ── Position ──────────────────────────────────────────────────────

  async openPosition(pool: PublicKey, params: OpenPositionParams): Promise<PublicKey> {
    const [posPda] = derivePositionAddress(this.id, pool, this.provider.wallet.publicKey);
    await this.program.methods
      .openPosition({ lowerBinId: params.lowerBinId, upperBinId: params.upperBinId } as never)
      .accounts({
        owner: this.provider.wallet.publicKey,
        pool,
        position: posPda,
        systemProgram: PublicKey.default,
      } as never)
      .rpc();
    return posPda;
  }

  async getPosition(address: PublicKey): Promise<PositionAccount | null> {
    try {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const acc = await (this.program.account as any).position.fetch(address);
      return acc as PositionAccount;
    } catch {
      return null;
    }
  }

  async addLiquidity(
    position: PublicKey,
    pool: PublicKey,
    deposits: BinDeposit[],
    binArrays: PublicKey[],
  ): Promise<string> {
    const tx = await this.program.methods
      .addLiquidity({ deposits: deposits as never } as never)
      .accounts({
        owner: this.provider.wallet.publicKey,
        position,
        pool,
        tokenVaultA: PublicKey.default,
        tokenVaultB: PublicKey.default,
        userTokenA: PublicKey.default,
        userTokenB: PublicKey.default,
        tokenProgram: PublicKey.default,
      } as never)
      .remainingAccounts(binArrays.map((pk) => ({ pubkey: pk, isWritable: true, isSigner: false })))
      .rpc();
    return tx;
  }

  async removeLiquidity(
    position: PublicKey,
    pool: PublicKey,
    deposits: BinDeposit[],
    binArrays: PublicKey[],
  ): Promise<string> {
    const tx = await this.program.methods
      .removeLiquidity({ deposits: deposits as never } as never)
      .accounts({
        owner: this.provider.wallet.publicKey,
        position,
        pool,
        tokenVaultA: PublicKey.default,
        tokenVaultB: PublicKey.default,
        userTokenA: PublicKey.default,
        userTokenB: PublicKey.default,
        tokenProgram: PublicKey.default,
      } as never)
      .remainingAccounts(binArrays.map((pk) => ({ pubkey: pk, isWritable: true, isSigner: false })))
      .rpc();
    return tx;
  }

  async collectFees(
    position: PublicKey,
    pool: PublicKey,
    binArrays: PublicKey[],
  ): Promise<string> {
    const tx = await this.program.methods
      .collectFees()
      .accounts({
        owner: this.provider.wallet.publicKey,
        position,
        pool,
        tokenVaultA: PublicKey.default,
        tokenVaultB: PublicKey.default,
        userTokenA: PublicKey.default,
        userTokenB: PublicKey.default,
        tokenProgram: PublicKey.default,
      } as never)
      .remainingAccounts(binArrays.map((pk) => ({ pubkey: pk, isWritable: true, isSigner: false })))
      .rpc();
    return tx;
  }

  // ── Swap ──────────────────────────────────────────────────────────

  async swap(
    pool: PublicKey,
    params: SwapParams,
    binArrays: PublicKey[],
  ): Promise<string> {
    const tx = await this.program.methods
      .swap({
        amount: params.amount,
        aToB: params.aToB,
        exactIn: params.exactIn,
        minAmountOut: params.minAmountOut,
        maxAmountIn: params.maxAmountIn,
        priceLimitBinId: params.priceLimitBinId,
      } as never)
      .accounts({
        payer: this.provider.wallet.publicKey,
        pool,
        tokenVaultA: PublicKey.default,
        tokenVaultB: PublicKey.default,
        userTokenA: PublicKey.default,
        userTokenB: PublicKey.default,
        tokenProgram: PublicKey.default,
      } as never)
      .remainingAccounts(binArrays.map((pk) => ({ pubkey: pk, isWritable: true, isSigner: false })))
      .rpc();
    return tx;
  }

  // ── Quoting ───────────────────────────────────────────────────────

  async quoteSwap(
    pool: PublicKey,
    params: Omit<QuoteParams, "binArrays">,
  ): Promise<SwapQuote> {
    const binArrays = await this.getBinArrays(pool);
    return quoteSwap({ ...params, binArrays });
  }
}
