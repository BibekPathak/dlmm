import assert from "assert";
import BN from "bn.js";
import * as fc from "fast-check";
import { PublicKey } from "@solana/web3.js";
import { quoteSwap } from "../sdk/quote";
import { BinArrayAccount, BINS_PER_ARRAY } from "../sdk/types";
import { createEmptyBinArray, fillBinArray, defaultPoolParams } from "./helpers/fixtures";

function makeBinArrays(activeBin: number, levels: number): BinArrayAccount[] {
  const pool = PublicKey.default;
  const start = Math.floor(activeBin / BINS_PER_ARRAY) * BINS_PER_ARRAY - levels;
  const end = Math.ceil(activeBin / BINS_PER_ARRAY) * BINS_PER_ARRAY + levels;
  const bas: BinArrayAccount[] = [];
  for (let s = start; s < end; s += BINS_PER_ARRAY) {
    const ba = createEmptyBinArray(pool, s);
    fillBinArray(ba, new BN("1000000000"), new BN("1000000000"));
    bas.push(ba);
  }
  return bas;
}

describe("quoteSwap — exactIn A→B", () => {
  it("should produce positive output for positive input", () => {
    fc.assert(
      fc.property(fc.integer({ min: 1000, max: 1_000_000 }), (amount) => {
        const { binStepBps, activeBinId, feeBps, priceLimitBinId } = defaultPoolParams();
        const bas = makeBinArrays(activeBinId, 100);
        const quote = quoteSwap({
          amount: new BN(amount),
          aToB: true,
          exactIn: true,
          feeBps,
          binStepBps,
          activeBinId,
          priceLimitBinId,
          binArrays: bas,
        });
        assert(quote.amountOut.gtn(0), `amount=${amount}: out should be > 0`);
        assert(quote.amountOut.lte(new BN(amount).muln(10)), `out too large: ${quote.amountOut.toString()}`);
        assert(quote.binsTraversed >= 0);
      }),
      { numRuns: 50 },
    );
  });

  it("should not exceed available liquidity", () => {
    const { binStepBps, activeBinId, feeBps, priceLimitBinId } = defaultPoolParams();
    const bas = makeBinArrays(activeBinId, 10);
    const totalY = bas.reduce((sum, ba) => {
      return sum.add(ba.bins.reduce((s, b) => s.add(b.amountY), new BN(0)));
    }, new BN(0));

    const quote = quoteSwap({
      amount: new BN("1000000000000"),
      aToB: true,
      exactIn: true,
      feeBps,
      binStepBps,
      activeBinId,
      priceLimitBinId,
      binArrays: bas,
    });
    assert(quote.amountOut.lte(totalY), `out ${quote.amountOut} > total Y ${totalY}`);
  });

  it("should respect price limit", () => {
    const { binStepBps, activeBinId, feeBps } = defaultPoolParams();
    const bas = makeBinArrays(activeBinId, 50);
    const limited = quoteSwap({
      amount: new BN(1_000_000),
      aToB: true,
      exactIn: true,
      feeBps,
      binStepBps,
      activeBinId,
      priceLimitBinId: activeBinId + 10,
      binArrays: bas,
    });
    const unlimited = quoteSwap({
      amount: new BN(1_000_000),
      aToB: true,
      exactIn: true,
      feeBps,
      binStepBps,
      activeBinId,
      priceLimitBinId: activeBinId + 1000,
      binArrays: bas,
    });
    assert(limited.amountOut.lte(unlimited.amountOut),
      `limited ${limited.amountOut} > unlimited ${unlimited.amountOut}`);
  });

  it("fee should be non-negative and <= amount * feeBps / 10000", () => {
    fc.assert(
      fc.property(fc.integer({ min: 1, max: 100_000 }), (amount) => {
        const { binStepBps, activeBinId, feeBps, priceLimitBinId } = defaultPoolParams();
        const bas = makeBinArrays(activeBinId, 50);
        const quote = quoteSwap({
          amount: new BN(amount),
          aToB: true,
          exactIn: true,
          feeBps,
          binStepBps,
          activeBinId,
          priceLimitBinId,
          binArrays: bas,
        });
        const maxFee = new BN(amount).muln(feeBps).divn(10000);
        assert(quote.fee.gte(new BN(0)), "fee negative");
        assert(quote.fee.lte(maxFee), `fee ${quote.fee} > max ${maxFee}`);
      }),
      { numRuns: 30 },
    );
  });
});

describe("quoteSwap — exactOut B→A", () => {
  it("should produce positive net input for reasonable output", () => {
    fc.assert(
      fc.property(fc.integer({ min: 5000, max: 100_000 }), (amount) => {
        const { binStepBps, activeBinId, feeBps } = defaultPoolParams();
        const bas = makeBinArrays(activeBinId, 50);
        const quote = quoteSwap({
          amount: new BN(amount),
          aToB: false,
          exactIn: false,
          feeBps,
          binStepBps,
          activeBinId,
          priceLimitBinId: -1000,
          binArrays: bas,
        });
        assert(quote.amountIn.gtn(0), `amount=${amount}: in should be > 0`);
      }),
      { numRuns: 30 },
    );
  });
});

describe("quoteSwap — roundtrip", () => {
  it("swap X→Y then swap back should approximately recover X", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 500, max: 10_000 }),
        fc.integer({ min: 1, max: 50 }),
        (amount, feeBps) => {
          const { binStepBps, activeBinId, priceLimitBinId } = defaultPoolParams();
          const bas = makeBinArrays(activeBinId, 100);

          const forward = quoteSwap({
            amount: new BN(amount),
            aToB: true,
            exactIn: true,
            feeBps,
            binStepBps,
            activeBinId,
            priceLimitBinId,
            binArrays: bas,
          });

          if (forward.amountOut.eqn(0)) return;

          const backward = quoteSwap({
            amount: forward.amountOut,
            aToB: false,
            exactIn: true,
            feeBps,
            binStepBps,
            activeBinId: activeBinId + forward.binsTraversed,
            priceLimitBinId: -1000,
            binArrays: bas,
          });

          const recovered = backward.amountOut;
          assert(recovered.lte(new BN(amount)),
            `recovered ${recovered} > original ${amount}`);
          assert(forward.fee.gte(new BN(0)), "forward fee negative");
          assert(backward.fee.gte(new BN(0)), "backward fee negative");
        },
      ),
      { numRuns: 30 },
    );
  });
});

describe("quoteSwap — edge cases", () => {
  it("should handle zero-fee swaps", () => {
    const { binStepBps, activeBinId, priceLimitBinId } = defaultPoolParams();
    const bas = makeBinArrays(activeBinId, 10);
    const quote = quoteSwap({
      amount: new BN(1000),
      aToB: true,
      exactIn: true,
      feeBps: 0,
      binStepBps,
      activeBinId,
      priceLimitBinId,
      binArrays: bas,
    });
    assert(quote.fee.eqn(0));
    assert(quote.amountIn.eq(quote.amountOut.add(quote.fee)));
  });

  it("should handle empty bin arrays gracefully", () => {
    const { binStepBps, activeBinId, feeBps, priceLimitBinId } = defaultPoolParams();
    const quote = quoteSwap({
      amount: new BN(1000),
      aToB: true,
      exactIn: true,
      feeBps,
      binStepBps,
      activeBinId,
      priceLimitBinId,
      binArrays: [],
    });
    assert(quote.amountOut.eqn(0));
    assert(quote.amountIn.eqn(0));
  });

  it("should handle single-bin swaps correctly", () => {
    const pool = PublicKey.default;
    const ba = createEmptyBinArray(pool, 0);
    ba.bins[0] = { amountX: new BN(5000), amountY: new BN(5000), feeX: new BN(0), feeY: new BN(0) };
    fillBinArray(ba, new BN(0), new BN(0), 1);

    const quote = quoteSwap({
      amount: new BN(100),
      aToB: true,
      exactIn: true,
      feeBps: 0,
      binStepBps: 10,
      activeBinId: 0,
      priceLimitBinId: 100,
      binArrays: [ba],
    });

    assert(quote.amountOut.gtn(0));
    assert(quote.binsTraversed >= 1);
  });

  it("should produce deterministic results", () => {
    const params = {
      amount: new BN(5000),
      aToB: true as const,
      exactIn: true as const,
      feeBps: 30,
      binStepBps: 10,
      activeBinId: 0,
      priceLimitBinId: 1000,
      binArrays: makeBinArrays(0, 50),
    };
    const q1 = quoteSwap(params);
    const q2 = quoteSwap(params);
    assert(q1.amountOut.eq(q2.amountOut));
    assert(q1.amountIn.eq(q2.amountIn));
    assert(q1.binsTraversed === q2.binsTraversed);
  });
});
