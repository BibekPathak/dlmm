import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

// ── Parameters ──────────────────────────────────────────────────────────

export interface InitializePoolParams {
  feeTierBps: number;
  protocolFeeBps: number;
  binStepBps: number;
  baseBinId: number;
  activeBinId: number;
  baseFeeBps: number;
  feeDecayInterval: BN;
}

export interface OpenPositionParams {
  lowerBinId: number;
  upperBinId: number;
}

export interface BinDeposit {
  binId: number;
  amountX: BN;
  amountY: BN;
}

export interface ModifyLiquidityParams {
  deposits: BinDeposit[];
}

export interface SwapParams {
  amount: BN;
  aToB: boolean;
  exactIn: boolean;
  minAmountOut: BN;
  maxAmountIn: BN;
  priceLimitBinId: number;
}

// ── On-chain account layouts ────────────────────────────────────────────

export interface PoolAccount {
  authority: PublicKey;
  tokenMintA: PublicKey;
  tokenMintB: PublicKey;
  tokenVaultA: PublicKey;
  tokenVaultB: PublicKey;
  feeTierBps: number;
  protocolFeeBps: number;
  binStepBps: number;
  baseBinId: number;
  activeBinId: number;
  pendingProtocolFeesX: BN;
  pendingProtocolFeesY: BN;
  baseFeeBps: number;
  variableFeeBps: number;
  volReferencePrice: BN;
  volAccumulator: BN;
  volLastTimestamp: BN;
  feeDecayInterval: BN;
  bump: number;
}

export interface Bin {
  amountX: BN;
  amountY: BN;
  feeX: BN;
  feeY: BN;
}

export interface BinArrayAccount {
  pool: PublicKey;
  startBinId: number;
  bins: Bin[];
}

export interface PositionAccount {
  owner: PublicKey;
  pool: PublicKey;
  lowerBinId: number;
  upperBinId: number;
  totalLiquidityX: BN;
  totalLiquidityY: BN;
  feeCheckpointX: BN;
  feeCheckpointY: BN;
  feesOwedX: BN;
  feesOwedY: BN;
  lastUpdate: BN;
  bump: number;
}

// ── Quote types ──────────────────────────────────────────────────────────

export interface SwapQuote {
  amountIn: BN;
  amountOut: BN;
  fee: BN;
  priceImpact: number;
  binsTraversed: number;
  route: number[];
}

export const BINS_PER_ARRAY = 64;
export const Q64 = new BN(1).shln(64);

export function binIdToArrayStart(binId: number): number {
  const rem = ((binId % BINS_PER_ARRAY) + BINS_PER_ARRAY) % BINS_PER_ARRAY;
  return binId - rem;
}
