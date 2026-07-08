import { PublicKey } from "@solana/web3.js";
import { binIdToArrayStart } from "./types";

export function derivePoolAddress(
  programId: PublicKey,
  mintA: PublicKey,
  mintB: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), mintA.toBuffer(), mintB.toBuffer()],
    programId,
  );
}

export function deriveBinArrayAddress(
  programId: PublicKey,
  pool: PublicKey,
  binId: number,
): [PublicKey, number] {
  const start = binIdToArrayStart(binId);
  const buf = Buffer.alloc(4);
  buf.writeInt32LE(start);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("bin_array"), pool.toBuffer(), buf],
    programId,
  );
}

export function derivePositionAddress(
  programId: PublicKey,
  pool: PublicKey,
  owner: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("position"), pool.toBuffer(), owner.toBuffer()],
    programId,
  );
}

export function deriveAllBinArrayAddresses(
  programId: PublicKey,
  pool: PublicKey,
  lowerBinId: number,
  upperBinId: number,
): [PublicKey, number][] {
  const start = binIdToArrayStart(lowerBinId);
  const end = binIdToArrayStart(upperBinId);
  const addrs: [PublicKey, number][] = [];
  for (let s = start; s <= end; s += 64) {
    addrs.push(deriveBinArrayAddress(programId, pool, s));
  }
  return addrs;
}
