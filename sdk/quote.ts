import BN from "bn.js";
import { PublicKey } from "@solana/web3.js";
import { BinArrayAccount, Bin, SwapQuote, Q64, BINS_PER_ARRAY } from "./types";

const BPS_DENOM = new BN(10_000);

function applyBps(amount: BN, bps: number): BN {
  return amount.muln(bps).div(BPS_DENOM);
}

function q64Mul(a: BN, b: BN): BN {
  // (a * b) / 2^64 using BN (which handles arbitrary precision)
  return a.mul(b).shrn(64);
}

function q64Div(a: BN, b: BN): BN {
  return a.shln(64).div(b);
}

function binToPrice(binId: number, stepBps: number): BN {
  const base = Q64.add(new BN(stepBps).mul(Q64).div(BPS_DENOM));
  const invBase = Q64.mul(BPS_DENOM).div(BPS_DENOM.addn(stepBps));

  const exp = Math.abs(binId);
  let result = new BN(1).shln(64);
  let b = binId >= 0 ? base : invBase;
  let e = exp;
  while (e > 0) {
    if (e & 1) result = q64Mul(result, b);
    b = q64Mul(b, b);
    e >>= 1;
  }
  return result;
}

function computeSwapStep(
  remainingNet: BN,
  bin: Bin,
  price: BN,
  aToB: boolean,
  feeBps: number,
): { amountInConsumed: BN; amountOut: BN; feePaid: BN; binDepleted: boolean } {
  if (aToB) {
    const available = bin.amountY;
    if (available.eqn(0)) {
      return { amountInConsumed: new BN(0), amountOut: new BN(0), feePaid: new BN(0), binDepleted: false };
    }
    const desiredOut = q64Mul(remainingNet, price);
    if (desiredOut.lte(available)) {
      return {
        amountInConsumed: remainingNet,
        amountOut: desiredOut,
        feePaid: applyBps(remainingNet, feeBps),
        binDepleted: false,
      };
    } else {
      const netNeeded = q64Div(available, price);
      return {
        amountInConsumed: netNeeded,
        amountOut: available,
        feePaid: applyBps(netNeeded, feeBps),
        binDepleted: true,
      };
    }
  } else {
    const available = bin.amountX;
    if (available.eqn(0)) {
      return { amountInConsumed: new BN(0), amountOut: new BN(0), feePaid: new BN(0), binDepleted: false };
    }
    const desiredOut = q64Div(remainingNet, price);
    if (desiredOut.lte(available)) {
      return {
        amountInConsumed: remainingNet,
        amountOut: desiredOut,
        feePaid: applyBps(remainingNet, feeBps),
        binDepleted: false,
      };
    } else {
      const netNeeded = q64Mul(available, price);
      return {
        amountInConsumed: netNeeded,
        amountOut: available,
        feePaid: applyBps(netNeeded, feeBps),
        binDepleted: true,
      };
    }
  }
}

function findBin(binArrays: BinArrayAccount[], binId: number): Bin | null {
  for (const ba of binArrays) {
    const offset = binId - ba.startBinId;
    if (offset >= 0 && offset < BINS_PER_ARRAY) {
      return ba.bins[offset];
    }
  }
  return null;
}

export interface QuoteParams {
  amount: BN;
  aToB: boolean;
  exactIn: boolean;
  feeBps: number;
  binStepBps: number;
  activeBinId: number;
  priceLimitBinId: number;
  binArrays: BinArrayAccount[];
}

export function quoteSwap(params: QuoteParams): SwapQuote {
  const { amount, aToB, exactIn, feeBps, binStepBps, activeBinId, priceLimitBinId, binArrays } = params;
  const direction = aToB ? 1 : -1;

  let totalNet = new BN(0);
  let totalOut = new BN(0);
  let totalFee = new BN(0);
  let curBinId = activeBinId;
  let binsTraversed = 0;

  if (exactIn) {
    let usedGross = new BN(0);
    while (true) {
      if ((aToB && curBinId >= priceLimitBinId) || (!aToB && curBinId <= priceLimitBinId)) break;
      if (usedGross.gte(amount)) break;

      const remainingGross = amount.sub(usedGross);
      const maxNet = remainingGross.mul(BPS_DENOM).div(BPS_DENOM.addn(feeBps));
      if (maxNet.eqn(0)) break;

      const bin = findBin(binArrays, curBinId);
      if (!bin) {
        curBinId += direction;
        binsTraversed++;
        continue;
      }

      const price = binToPrice(curBinId, binStepBps);
      const step = computeSwapStep(maxNet, bin, price, aToB, feeBps);
      if (step.amountOut.eqn(0) && !step.binDepleted) {
        curBinId += direction;
        binsTraversed++;
        continue;
      }

      totalNet = totalNet.add(step.amountInConsumed);
      totalOut = totalOut.add(step.amountOut);
      totalFee = totalFee.add(step.feePaid);
      binsTraversed++;

      const stepGross = step.amountInConsumed.add(step.feePaid);
      usedGross = usedGross.add(stepGross);

      if (step.binDepleted) {
        curBinId += direction;
      } else {
        break;
      }
    }
  } else {
    let remainingOut = amount;
    while (true) {
      if ((aToB && curBinId >= priceLimitBinId) || (!aToB && curBinId <= priceLimitBinId)) break;
      if (remainingOut.eqn(0)) break;

      const bin = findBin(binArrays, curBinId);
      if (!bin) {
        curBinId += direction;
        binsTraversed++;
        continue;
      }

      const price = binToPrice(curBinId, binStepBps);
      const available = aToB ? bin.amountY : bin.amountX;

      if (available.eqn(0)) {
        curBinId += direction;
        binsTraversed++;
        continue;
      }

      let netNeeded: BN;
      let outThisStep: BN;
      if (aToB) {
        netNeeded = q64Div(remainingOut, price);
        outThisStep = remainingOut.lte(available) ? remainingOut : available;
      } else {
        netNeeded = q64Mul(remainingOut, price);
        outThisStep = remainingOut.lte(available) ? remainingOut : available;
      }

      const fee = applyBps(netNeeded, feeBps);
      totalNet = totalNet.add(netNeeded);
      totalOut = totalOut.add(outThisStep);
      totalFee = totalFee.add(fee);
      binsTraversed++;

      if (outThisStep.eq(remainingOut)) {
        remainingOut = new BN(0);
      } else {
        remainingOut = remainingOut.sub(outThisStep);
        curBinId += direction;
      }
    }
  }

  const amountIn = totalNet.add(totalFee);
  const priceImpact = amountIn.gtn(0)
    ? totalOut.muln(10_000).div(amountIn).toNumber() / 100
    : 0;

  return {
    amountIn,
    amountOut: totalOut,
    fee: totalFee,
    priceImpact,
    binsTraversed,
    route: [],
  };
}
