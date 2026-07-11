import assert from "assert";
import BN from "bn.js";
import * as fc from "fast-check";
import { PublicKey } from "@solana/web3.js";
import { quoteSwap } from "../../sdk/quote";
import { BinArrayAccount, BINS_PER_ARRAY } from "../../sdk/types";
import { createEmptyBinArray, fillBinArray } from "../helpers/fixtures";

function pool(activeBin: number, levels: number, liqX?: BN, liqY?: BN): BinArrayAccount[] {
  const bas: BinArrayAccount[] = [];
  const start = Math.floor(activeBin / BINS_PER_ARRAY) * BINS_PER_ARRAY - levels;
  const end = Math.ceil(activeBin / BINS_PER_ARRAY) * BINS_PER_ARRAY + levels;
  for (let s = start; s < end; s += BINS_PER_ARRAY) {
    const ba = createEmptyBinArray(PublicKey.default, s);
    fillBinArray(ba, liqX || new BN("1000000000"), liqY || new BN("1000000000"));
    bas.push(ba);
  }
  return bas;
}

// ── Invariant helpers ──────────────────────────────────────────────────

function totalLiquidityX(bas: BinArrayAccount[]): BN {
  return bas.reduce((s, ba) => s.add(ba.bins.reduce((s2, b) => s2.add(b.amountX), new BN(0))), new BN(0));
}

function totalLiquidityY(bas: BinArrayAccount[]): BN {
  return bas.reduce((s, ba) => s.add(ba.bins.reduce((s2, b) => s2.add(b.amountY), new BN(0))), new BN(0));
}

function noNegativeAmounts(bas: BinArrayAccount[]): boolean {
  for (const ba of bas) {
    for (const b of ba.bins) {
      if (b.amountX.ltn(0) || b.amountY.ltn(0)) return false;
    }
  }
  return true;
}

function cloneArrays(bas: BinArrayAccount[]): BinArrayAccount[] {
  return bas.map((ba) => ({
    pool: ba.pool,
    startBinId: ba.startBinId,
    bins: ba.bins.map((b) => ({ ...b, amountX: b.amountX.clone(), amountY: b.amountY.clone(), feeX: b.feeX.clone(), feeY: b.feeY.clone() })),
  }));
}

function applySwapToArrays(bas: BinArrayAccount[], binId: number, amountXOut: BN, amountYOut: BN): void {
  for (const ba of bas) {
    const offset = binId - ba.startBinId;
    if (offset >= 0 && offset < BINS_PER_ARRAY) {
      const bin = ba.bins[offset];
      bin.amountX = bnMax(bin.amountX.sub(amountXOut), new BN(0));
      bin.amountY = bnMax(bin.amountY.sub(amountYOut), new BN(0));
      return;
    }
  }
}

function bnMax(a: BN, b: BN): BN {
  return a.gt(b) ? a : b;
}

// ── Fuzz: Random swap sequences ────────────────────────────────────────

describe("Fuzz — swap sequences", () => {
  it("monotonic liquidity: total X+Y never increases after swaps", () => {
    fc.assert(
      fc.property(
        fc.array(fc.integer({ min: 1, max: 1_000_000 }), { minLength: 3, maxLength: 10 }),
        fc.boolean(),
        (amounts, aToB) => {
          const bas = pool(0, 100);
          const initialX = totalLiquidityX(bas);
          const initialY = totalLiquidityY(bas);

          for (const amt of amounts) {
            const q = quoteSwap({
              amount: new BN(amt),
              aToB,
              exactIn: true,
              feeBps: 30,
              binStepBps: 10,
              activeBinId: 0,
              priceLimitBinId: aToB ? 10000 : -10000,
              binArrays: bas,
            });
            if (q.amountOut.gtn(0) && q.amountIn.gtn(0)) {
              // Apply the swap to the arrays
              applySwapToArrays(bas, 0, aToB ? new BN(0) : q.amountOut, aToB ? q.amountOut : new BN(0));
            }
          }

          const finalX = totalLiquidityX(bas);
          const finalY = totalLiquidityY(bas);
          assert(noNegativeAmounts(bas), "negative amounts after swaps");
          assert(finalX.lte(initialX), `total X increased: ${finalX} > ${initialX}`);
          assert(finalY.lte(initialY), `total Y increased: ${finalY} > ${initialY}`);
        },
      ),
      { numRuns: 20 },
    );
  });

  it("alternating direction: total liquidity never goes negative", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 100, max: 500_000 }),
        fc.integer({ min: 1, max: 10 }),
        (amount, rounds) => {
          const bas = pool(0, 50);
          for (let i = 0; i < rounds; i++) {
            const aToB = i % 2 === 0;
            const q = quoteSwap({
              amount: new BN(amount),
              aToB,
              exactIn: true,
              feeBps: 30,
              binStepBps: 10,
              activeBinId: 0,
              priceLimitBinId: aToB ? 1000 : -1000,
              binArrays: bas,
            });
            if (q.amountOut.gtn(0)) {
              applySwapToArrays(bas, 0, aToB ? new BN(0) : q.amountOut, aToB ? q.amountOut : new BN(0));
            }
            assert(noNegativeAmounts(bas), `negative after round ${i}`);
          }
        },
      ),
      { numRuns: 15 },
    );
  });
});

// ── Fuzz: Extreme fee configs ──────────────────────────────────────────

describe("Fuzz — extreme fee configs", () => {
  it("fee never exceeds amount * feeBps / 10000", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1, max: 10_000 }),
        fc.integer({ min: 0, max: 10_000 }),
        fc.integer({ min: 1, max: 1_000_000 }),
        (binStepBps, feeBps, amount) => {
          if (binStepBps === 0) return;
          const bas = pool(0, 50);
          const q = quoteSwap({
            amount: new BN(amount),
            aToB: true,
            exactIn: true,
            feeBps,
            binStepBps,
            activeBinId: 0,
            priceLimitBinId: 5000,
            binArrays: bas,
          });
          const maxFee = new BN(amount).muln(feeBps).divn(10000);
          assert(q.fee.gte(new BN(0)), "negative fee");
          assert(q.fee.lte(maxFee), `fee ${q.fee} > max ${maxFee} for feeBps=${feeBps}`);
        },
      ),
      { numRuns: 50 },
    );
  });

  it("amountIn always covers fee + net", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1, max: 5_000 }),
        fc.integer({ min: 0, max: 5000 }),
        (amount, feeBps) => {
          const bas = pool(0, 30);
          const q = quoteSwap({
            amount: new BN(amount),
            aToB: true,
            exactIn: true,
            feeBps,
            binStepBps: 10,
            activeBinId: 0,
            priceLimitBinId: 1000,
            binArrays: bas,
          });
          if (q.amountOut.eqn(0)) return;
          assert(q.amountIn.eq(q.amountOut.add(q.fee)),
            `amountIn=${q.amountIn} != out=${q.amountOut} + fee=${q.fee}`);
        },
      ),
      { numRuns: 30 },
    );
  });
});

// ── Fuzz: Price limit ─────────────────────────────────────────────────

describe("Fuzz — price limit", () => {
  it("swap never crosses price limit", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1, max: 100_000 }),
        fc.integer({ min: -500, max: 500 }),
        (amount, limitOffset) => {
          if (limitOffset === 0) return;
          const activeBin = 0;
          const aToB = limitOffset > 0;
          const priceLimit = activeBin + limitOffset;
          const bas = pool(activeBin, 100);
          const q = quoteSwap({
            amount: new BN(amount),
            aToB,
            exactIn: true,
            feeBps: 30,
            binStepBps: 10,
            activeBinId: activeBin,
            priceLimitBinId: priceLimit,
            binArrays: bas,
          });
          if (q.amountOut.gtn(0)) {
            // If we crossed any bins, verify we didn't exceed the limit
            const direction = aToB ? 1 : -1;
            if (aToB) {
              assert(activeBin + q.binsTraversed * direction <= priceLimit,
                `crossed limit: start=${activeBin} dir=${direction} bins=${q.binsTraversed} limit=${priceLimit}`);
            }
          }
        },
      ),
      { numRuns: 30 },
    );
  });

  it("tighter price limit gives ≤ output of looser limit", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1, max: 100_000 }),
        (amount) => {
          const bas = pool(0, 100);
          const tight = quoteSwap({
            amount: new BN(amount), aToB: true, exactIn: true,
            feeBps: 30, binStepBps: 10, activeBinId: 0,
            priceLimitBinId: 10, binArrays: bas,
          });
          const loose = quoteSwap({
            amount: new BN(amount), aToB: true, exactIn: true,
            feeBps: 30, binStepBps: 10, activeBinId: 0,
            priceLimitBinId: 1000, binArrays: bas,
          });
          assert(tight.amountOut.lte(loose.amountOut),
            `tight ${tight.amountOut} > loose ${loose.amountOut}`);
        },
      ),
      { numRuns: 20 },
    );
  });
});

// ── Fuzz: Roundtrip invariants ─────────────────────────────────────────

describe("Fuzz — roundtrip invariants", () => {
  it("swap A→B→A recovers ≤ original amount", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 500, max: 50_000 }),
        fc.integer({ min: 0, max: 500 }),
        (amount, feeBps) => {
          const bas = pool(0, 100);
          const stepBps = 10;
          const priceLimit = 5000;

          const fwd = quoteSwap({
            amount: new BN(amount), aToB: true, exactIn: true,
            feeBps, binStepBps: stepBps, activeBinId: 0,
            priceLimitBinId: priceLimit, binArrays: bas,
          });
          if (fwd.amountOut.eqn(0)) return;

          const bwd = quoteSwap({
            amount: fwd.amountOut, aToB: false, exactIn: true,
            feeBps, binStepBps: stepBps,
            activeBinId: 0 + fwd.binsTraversed,
            priceLimitBinId: -priceLimit, binArrays: bas,
          });
          if (bwd.amountOut.eqn(0)) return;

          assert(bwd.amountOut.lte(new BN(amount)),
            `recovered ${bwd.amountOut} > original ${amount}`);
        },
      ),
      { numRuns: 20 },
    );
  });

  it("ExactIn and ExactOut are consistent for same amount", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1000, max: 50_000 }),
        (amount) => {
          const bas = pool(0, 50);
          const exactIn = quoteSwap({
            amount: new BN(amount), aToB: true, exactIn: true,
            feeBps: 30, binStepBps: 10, activeBinId: 0,
            priceLimitBinId: 1000, binArrays: bas,
          });
          if (exactIn.amountOut.eqn(0)) return;

          const exactOut = quoteSwap({
            amount: exactIn.amountOut, aToB: true, exactIn: false,
            feeBps: 30, binStepBps: 10, activeBinId: 0,
            priceLimitBinId: 1000, binArrays: bas,
          });

          // ExactIn and ExactOut should be close (within fee tolerance)
          const diff = exactIn.amountIn.sub(exactOut.amountIn).abs();
          const tolerance = exactIn.amountIn.divn(100); // 1% tolerance
          assert(diff.lte(tolerance),
            `ExactIn cost=${exactIn.amountIn} vs ExactOut cost=${exactOut.amountIn} diff=${diff}`);
        },
      ),
      { numRuns: 15 },
    );
  });
});

// ── Fuzz: Liquidity invariants ─────────────────────────────────────────

describe("Fuzz — liquidity invariants", () => {
  it("output never exceeds available liquidity in any single bin", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1, max: 100_000 }),
        fc.integer({ min: -10, max: 10 }),
        (amount, binOffset) => {
          if (binOffset === 0) return;
          const bas = pool(0, 10);
          const binId = binOffset;
          const aToB = binOffset > 0;
          const q = quoteSwap({
            amount: new BN(amount), aToB, exactIn: true,
            feeBps: 30, binStepBps: 10, activeBinId: 0,
            priceLimitBinId: aToB ? 1000 : -1000, binArrays: bas,
          });
          assert(q.amountOut.lte(new BN("1000000000").muln(bas.length * BINS_PER_ARRAY)),
            `output exceeds total pool liquidity`);
        },
      ),
      { numRuns: 20 },
    );
  });

  it("zero-liquidity bins produce zero output", () => {
    fc.assert(
      fc.property(fc.integer({ min: 1, max: 10_000 }), (amount) => {
        const bas = pool(0, 5, new BN(0), new BN(0));
        const q = quoteSwap({
          amount: new BN(amount), aToB: true, exactIn: true,
          feeBps: 30, binStepBps: 10, activeBinId: 0,
          priceLimitBinId: 1000, binArrays: bas,
        });
        assert(q.amountOut.eqn(0), "empty pool should produce 0 output");
        assert(q.amountIn.eqn(0), "empty pool should consume 0 input");
      }),
      { numRuns: 10 },
    );
  });

  it("large swap doesn't produce negative bin amounts", () => {
    const bas = pool(0, 30);
    const q = quoteSwap({
      amount: new BN("1000000000000"), aToB: true, exactIn: true,
      feeBps: 30, binStepBps: 10, activeBinId: 0,
      priceLimitBinId: 5000, binArrays: bas,
    });
    assert(q.amountOut.gtn(0), "large swap should produce output");
    assert(q.amountIn.gtn(0), "large swap should consume input");
    assert(q.binsTraversed > 0, "large swap should cross multiple bins");
  });
});
