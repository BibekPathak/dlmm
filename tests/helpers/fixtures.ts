import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { BinArrayAccount, Bin, BINS_PER_ARRAY } from "../../sdk/types";

export function createEmptyBin(): Bin {
  return { amountX: new BN(0), amountY: new BN(0), feeX: new BN(0), feeY: new BN(0) };
}

export function createEmptyBinArray(pool: PublicKey, startBinId: number): BinArrayAccount {
  const bins: Bin[] = [];
  for (let i = 0; i < BINS_PER_ARRAY; i++) {
    bins.push(createEmptyBin());
  }
  return { pool, startBinId, bins };
}

export function fillBinArray(
  ba: BinArrayAccount,
  amountX: BN,
  amountY: BN,
  startOffset?: number,
  count?: number,
): void {
  const s = startOffset ?? 0;
  const c = count ?? BINS_PER_ARRAY;
  for (let i = s; i < Math.min(s + c, BINS_PER_ARRAY); i++) {
    ba.bins[i] = { amountX, amountY, feeX: new BN(0), feeY: new BN(0) };
  }
}

export function defaultPoolParams() {
  return {
    binStepBps: 10,
    activeBinId: 0,
    feeBps: 30,
    priceLimitBinId: 1000,
  };
}
