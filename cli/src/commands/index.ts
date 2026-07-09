import { Command } from "commander";
import { PublicKey, Keypair, Connection } from "@solana/web3.js";
import { AnchorProvider, Wallet } from "@coral-xyz/anchor";
import { DlmmClient } from "../../../sdk/DlmmClient";
import {
  derivePoolAddress,
  deriveBinArrayAddress,
  deriveAllBinArrayAddresses,
} from "../../../sdk/pda";
import { quoteSwap } from "../../../sdk/quote";
import { readFileSync } from "fs";
import { homedir } from "os";
import { join } from "path";
import BN from "bn.js";

function loadKeypair(path?: string): Keypair {
  const p = path || join(homedir(), ".config", "solana", "id.json");
  return Keypair.fromSecretKey(new Uint8Array(JSON.parse(readFileSync(p, "utf-8"))));
}

function provider(rpcUrl?: string, keypairPath?: string): AnchorProvider {
  const url = rpcUrl || process.env.ANCHOR_PROVIDER_URL || "http://127.0.0.1:8899";
  return new AnchorProvider(
    new Connection(url, "confirmed"),
    new Wallet(loadKeypair(keypairPath)),
    { commitment: "confirmed" },
  );
}

function client(prov: AnchorProvider, id?: string): DlmmClient {
  return new DlmmClient(
    prov,
    new PublicKey(id || process.env.DLMM_PROGRAM_ID || "So11111111111111111111111111111111111111112"),
  );
}

export function registerCommands(prog: Command) {
  prog
    .command("create-pool")
    .description("Create a liquidity pool")
    .requiredOption("--mint-a <addr>", "Token A mint")
    .requiredOption("--mint-b <addr>", "Token B mint")
    .requiredOption("--bin-step <n>", "Bin step in bps")
    .requiredOption("--base-fee <n>", "Base fee in bps")
    .option("--protocol-fee <n>", "Protocol fee bps", "0")
    .option("--active-bin <n>", "Active bin ID", "0")
    .option("--decay <n>", "Decay interval sec", "3600")
    .option("--keypair <path>", "Keypair path")
    .option("--rpc <url>", "RPC URL")
    .option("--program-id <addr>", "Program ID")
    .action(async (opts) => {
      const prov = provider(opts.rpc, opts.keypair);
      const c = client(prov, opts.programId);
      const mintA = new PublicKey(opts.mintA);
      const mintB = new PublicKey(opts.mintB);

      const sig = await c.createPool(prov.wallet.publicKey, mintA, mintB, {
        feeTierBps: Number(opts.baseFee),
        protocolFeeBps: Number(opts.protocolFee),
        binStepBps: Number(opts.binStep),
        baseBinId: 0,
        activeBinId: Number(opts.activeBin),
        baseFeeBps: Number(opts.baseFee),
        feeDecayInterval: new BN(opts.decay),
      });
      const [pda] = derivePoolAddress(c.id, mintA, mintB);
      console.log("Pool:", pda.toBase58());
      console.log("Sig:", sig);
    });

  prog
    .command("init-ba")
    .description("Initialize a BinArray")
    .requiredOption("--pool <addr>", "Pool address")
    .requiredOption("--start-bin <n>", "Start bin ID")
    .option("--keypair <path>", "Keypair path")
    .option("--rpc <url>", "RPC URL")
    .option("--program-id <addr>", "Program ID")
    .action(async (opts) => {
      const prov = provider(opts.rpc, opts.keypair);
      const c = client(prov, opts.programId);
      const pool = new PublicKey(opts.pool);
      const sig = await c.createBinArray(pool, Number(opts.startBin));
      const [baPda] = deriveBinArrayAddress(c.id, pool, Number(opts.startBin));
      console.log("BinArray:", baPda.toBase58());
      console.log("Sig:", sig);
    });

  prog
    .command("open-pos")
    .description("Open a position")
    .requiredOption("--pool <addr>", "Pool address")
    .requiredOption("--lower <n>", "Lower bin")
    .requiredOption("--upper <n>", "Upper bin")
    .option("--keypair <path>", "Keypair path")
    .option("--rpc <url>", "RPC URL")
    .option("--program-id <addr>", "Program ID")
    .action(async (opts) => {
      const prov = provider(opts.rpc, opts.keypair);
      const c = client(prov, opts.programId);
      const pool = new PublicKey(opts.pool);
      const pos = await c.openPosition(pool, {
        lowerBinId: Number(opts.lower),
        upperBinId: Number(opts.upper),
      });
      console.log("Position:", pos.toBase58());
    });

  prog
    .command("add-liq")
    .description("Add liquidity")
    .requiredOption("--position <addr>", "Position")
    .requiredOption("--pool <addr>", "Pool")
    .requiredOption("--bins <ids>", "Comma-sep bin IDs")
    .requiredOption("--x <vals>", "Comma-sep X amounts")
    .requiredOption("--y <vals>", "Comma-sep Y amounts")
    .option("--keypair <path>")
    .option("--rpc <url>")
    .option("--program-id <addr>")
    .action(async (opts) => {
      const prov = provider(opts.rpc, opts.keypair);
      const c = client(prov, opts.programId);
      const position = new PublicKey(opts.position);
      const pool = new PublicKey(opts.pool);
      const bins = opts.bins.split(",").map(Number);
      const xs = opts.x.split(",").map((s: string) => new BN(s));
      const ys = opts.y.split(",").map((s: string) => new BN(s));
      const deposits = bins.map((b: number, i: number) => ({ binId: b, amountX: xs[i], amountY: ys[i] }));
      const bas = bins.map((b: number) => deriveBinArrayAddress(c.id, pool, b)[0]);
      const sig = await c.addLiquidity(position, pool, deposits, bas);
      console.log("Sig:", sig);
    });

  prog
    .command("rm-liq")
    .description("Remove liquidity")
    .requiredOption("--position <addr>")
    .requiredOption("--pool <addr>")
    .requiredOption("--bins <ids>")
    .requiredOption("--x <vals>")
    .requiredOption("--y <vals>")
    .option("--keypair <path>")
    .option("--rpc <url>")
    .option("--program-id <addr>")
    .action(async (opts) => {
      const prov = provider(opts.rpc, opts.keypair);
      const c = client(prov, opts.programId);
      const bins = opts.bins.split(",").map(Number);
      const xs = opts.x.split(",").map((s: string) => new BN(s));
      const ys = opts.y.split(",").map((s: string) => new BN(s));
      const deposits = bins.map((b: number, i: number) => ({ binId: b, amountX: xs[i], amountY: ys[i] }));
      const bas = bins.map((b: number) => deriveBinArrayAddress(c.id, new PublicKey(opts.pool), b)[0]);
      const sig = await c.removeLiquidity(
        new PublicKey(opts.position), new PublicKey(opts.pool), deposits, bas,
      );
      console.log("Sig:", sig);
    });

  prog
    .command("swap")
    .description("Execute a swap")
    .requiredOption("--pool <addr>")
    .requiredOption("--amount <n>")
    .requiredOption("--dir <d>", "a-to-b or b-to-a")
    .option("--exact-out", "ExactOut mode")
    .option("--min-out <n>", "Min output", "1")
    .option("--max-in <n>", "Max input")
    .option("--price-limit <n>", "Price limit bin")
    .option("--keypair <path>")
    .option("--rpc <url>")
    .option("--program-id <addr>")
    .action(async (opts) => {
      const prov = provider(opts.rpc, opts.keypair);
      const c = client(prov, opts.programId);
      const pool = new PublicKey(opts.pool);
      const aToB = opts.dir === "a-to-b";
      const acc = await c.getPool(pool);
      if (!acc) throw new Error("Pool not found");
      const d = aToB ? 1 : -1;
      const limit = opts.priceLimit ? Number(opts.priceLimit) : (aToB ? 2147483647 : -2147483648);
      const bas = deriveAllBinArrayAddresses(c.id, pool, acc.activeBinId, acc.activeBinId + d * 100)
        .map((p: [PublicKey, number]) => p[0]);
      const sig = await c.swap(pool, {
        amount: new BN(opts.amount),
        aToB,
        exactIn: !opts.exactOut,
        minAmountOut: new BN(opts.minOut),
        maxAmountIn: new BN(opts.maxIn || opts.amount),
        priceLimitBinId: limit,
      }, bas);
      console.log("Sig:", sig);
    });

  prog
    .command("quote")
    .description("Get a swap quote")
    .requiredOption("--pool <addr>")
    .requiredOption("--amount <n>")
    .requiredOption("--dir <d>", "a-to-b or b-to-a")
    .option("--fee-bps <n>", "Fee bps", "30")
    .option("--bin-step <n>", "Bin step", "10")
    .option("--active-bin <n>", "Active bin", "0")
    .option("--exact-out", "ExactOut mode")
    .option("--price-limit <n>")
    .option("--rpc <url>")
    .option("--program-id <addr>")
    .action(async (opts) => {
      const prov = provider(opts.rpc);
      const c = client(prov, opts.programId);
      const pool = new PublicKey(opts.pool);
      const aToB = opts.dir === "a-to-b";
      const binArrays = await c.getBinArrays(pool);
      const limit = opts.priceLimit
        ? Number(opts.priceLimit)
        : (aToB ? 2147483647 : -2147483648);

      const q = quoteSwap({
        amount: new BN(opts.amount),
        aToB,
        exactIn: !opts.exactOut,
        feeBps: Number(opts.feeBps),
        binStepBps: Number(opts.binStep),
        activeBinId: Number(opts.activeBin),
        priceLimitBinId: limit,
        binArrays,
      });

      const Table = (await import("cli-table3")).default;
      const t = new Table();
      t.push(["In", q.amountIn.toString(10)]);
      t.push(["Out", q.amountOut.toString(10)]);
      t.push(["Fee", q.fee.toString(10)]);
      t.push(["Bins", String(q.binsTraversed)]);
      t.push(["Impact", `${q.priceImpact.toFixed(2)}%`]);
      console.log(t.toString());
    });

  prog
    .command("collect")
    .description("Collect fees from a position")
    .requiredOption("--position <addr>")
    .requiredOption("--pool <addr>")
    .option("--keypair <path>")
    .option("--rpc <url>")
    .option("--program-id <addr>")
    .action(async (opts) => {
      const prov = provider(opts.rpc, opts.keypair);
      const c = client(prov, opts.programId);
      const position = new PublicKey(opts.position);
      const pool = new PublicKey(opts.pool);
      const posAcc = await c.getPosition(position);
      if (!posAcc) throw new Error("Position not found");
      const bas = deriveAllBinArrayAddresses(c.id, pool, posAcc.lowerBinId, posAcc.upperBinId)
        .map((p: [PublicKey, number]) => p[0]);
      const sig = await c.collectFees(position, pool, bas);
      console.log("Sig:", sig);
    });

  prog
    .command("show-pool")
    .description("Show pool details")
    .requiredOption("--pool <addr>")
    .option("--rpc <url>")
    .option("--program-id <addr>")
    .action(async (opts) => {
      const prov = provider(opts.rpc);
      const c = client(prov, opts.programId);
      const pool = new PublicKey(opts.pool);
      const a = await c.getPool(pool);
      if (!a) throw new Error("Pool not found");
      const Table = (await import("cli-table3")).default;
      const t = new Table();
      t.push(["Mint A", a.tokenMintA.toBase58()]);
      t.push(["Mint B", a.tokenMintB.toBase58()]);
      t.push(["Active Bin", String(a.activeBinId)]);
      t.push(["Base Fee", `${a.baseFeeBps} bps`]);
      t.push(["Variable Fee", `${a.variableFeeBps} bps`]);
      console.log(t.toString());
    });

  prog
    .command("show-pos")
    .description("Show position details")
    .requiredOption("--position <addr>")
    .option("--rpc <url>")
    .option("--program-id <addr>")
    .action(async (opts) => {
      const prov = provider(opts.rpc);
      const c = client(prov, opts.programId);
      const p = new PublicKey(opts.position);
      const a = await c.getPosition(p);
      if (!a) throw new Error("Position not found");
      const Table = (await import("cli-table3")).default;
      const t = new Table();
      t.push(["Pool", a.pool.toBase58()]);
      t.push(["Range", `[${a.lowerBinId}, ${a.upperBinId}]`]);
      t.push(["Liq X", a.totalLiquidityX.toString(10)]);
      t.push(["Liq Y", a.totalLiquidityY.toString(10)]);
      t.push(["Fees X", a.feesOwedX.toString(10)]);
      t.push(["Fees Y", a.feesOwedY.toString(10)]);
      console.log(t.toString());
    });
}
