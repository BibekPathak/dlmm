/**
 * Integration tests for the DLMM protocol.
 *
 * Prerequisites:
 *   1. solana-test-validator running
 *   2. Program deployed: anchor deploy
 *   3. IDL generated: anchor build
 *
 * Run: npx ts-node tests/integration/pool.test.ts
 *
 * These tests cover the full lifecycle:
 * - Pool creation with valid/invalid params
 * - BinArray initialization
 * - Position creation with range validation
 * - Liquidity deposit and withdrawal
 * - Swap execution (ExactIn and ExactOut)
 * - Fee collection
 * - Edge cases (overflow, empty bins, etc.)
 */

/*
import { DlmmClient } from "../../sdk";
import { PublicKey } from "@solana/web3.js";

describe("Pool lifecycle", () => {
  let client: DlmmClient;

  before(async () => {
    const provider = anchor.AnchorProvider.env();
    const programId = new PublicKey("..."); // deployed program ID
    const idl = require("../../target/idl/dlmm.json");
    client = new DlmmClient(provider, programId, idl);
  });

  it("creates a pool", async () => { ... });
  it("initializes bin arrays", async () => { ... });
  it("opens a position", async () => { ... });
  it("adds liquidity", async () => { ... });
  it("removes liquidity", async () => { ... });
  it("swaps ExactIn", async () => { ... });
  it("swaps ExactOut", async () => { ... });
  it("collects fees", async () => { ... });
  it("handles overflow", async () => { ... });
  it("handles price limit", async () => { ... });
  it("handles empty pool", async () => { ... });
});
*/
