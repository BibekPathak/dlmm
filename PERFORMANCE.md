# DLMM Performance

## Real Benchmarks (x86_64 host)

Measured with `std::time::Instant` across 1M iterations (fixed-point)
or 100K iterations (price/swap). Compiled with nightly Rust benchmark profile.

Absolute numbers will be ~20× higher on Solana BPF VM, but relative costs
between operations are representative. Estimates based on published
Solana compute costs: ~1 µs x86 ≈ ~20 µs SBF (20× slowdown for
integer arithmetic in a constrained VM).

### Fixed-Point Arithmetic

| Operation | Input | ns/iter | Notes |
|-----------|-------|---------|-------|
| `q64_mul` | Q64 × Q64 | **416** | Widening multiply (64-bit limb decomposition) |
| `q64_div` | (Q64/2) ÷ Q64 | **152** | |
| `base_multiplier` | step=10 bps | **0** | Constant-time arithmetic, essentially free |
| `inv_base_multiplier` | step=10 bps | **0** | |

### Price Computation

| Operation | Input | ns/iter | Notes |
|-----------|-------|---------|-------|
| `bin_to_price` | bin=0 | **8** | Trivial: returns Q64 immediately |
| `bin_to_price` | bin=100 | **4,233** | `pow_q64(base, 100)` — log₂(100) = 7 iterations |
| `bin_to_price` | bin=1,000 | **6,186** | log₂(1K) = 10 iterations |
| `bin_to_price` | bin=10,000 | **8,068** | log₂(10K) = 14 iterations |
| `bin_to_price` | bin=-1,000 | **6,624** | Uses `inv_base`, symmetric cost |
| `pow_q64` | exp=10 | **2,436** | Binary exponentiation, 4 iterations |
| `pow_q64` | exp=100 | **4,188** | 7 iterations |
| `pow_q64` | exp=1,000 | **6,834** | 10 iterations |

**Key insight:** Binary exponentiation scales as O(log n). Going from bin=100
to bin=10,000 (100× increase) adds only ~3.8 µs (~90% increase). Iterative
multiplication would take 100× longer.

### Swap Step Computation

| Operation | ns/iter | Notes |
|-----------|---------|-------|
| Partial fill | **221** | Compute desired output via `q64_mul`, bin stays non-empty |
| Full depletion | **380** | Compute net needed to drain bin via `q64_div` |
| Empty bin (skip) | **8** | Immediate return via zero-check |

### Projected Swap Latency (x86_64 host)

| Bins Traversed | Computation Time | Formula |
|----------------|-----------------|---------|
| 1 | **4.6 µs** | `bin_to_price(bin)` + `swap_step` |
| 10 | **10.7 µs** | + 9 × (q64_mul + swap_step) |
| 50 | **36.5 µs** | |
| 100 | **69 µs** | |
| 200 | **134 µs** | |
| 500 | **332 µs** | |

**Formula:** T(n) = bin_to_price(initial) + n × (q64_mul + swap_step_avg)

Estimated on Solana BPF (~20×): 100-bin swap ≈ **1.4 ms**, 500-bin swap ≈ **6.6 ms**.

## Account Sizes

| Account | Size | Zero-Copy | Notes |
|---------|------|-----------|-------|
| Pool | **201 B** | No | Single account per pool pair |
| Position | **145 B** | No | One per user per pool |
| BinArray | **2,096 B** | **Yes** | One per 64-bin range, zero-copy bytemuck |
| Bin (within) | **32 B** | **Yes** | 4 × u64, direct memory mapping |

## Gas Optimization Techniques

1. **Zero-copy accounts** — `#[account(zero_copy)]` + `#[repr(C)]` + bytemuck
   `from_bytes_mut` eliminates Borsh deserialization overhead on every swap
   iteration.

2. **Incremental price computation** — O(log n) binary exponentiation for
   the first bin (4.2 µs), then O(1) `q64_mul(price, step_multiplier)` per
   subsequent bin (0.42 µs) — **10× faster** per bin than recomputing
   `bin_to_price` for each bin.

3. **Per-BinArray iteration** — Outer loop processes one BinArray (64 bins)
   at a time, avoiding PDA lookups.

4. **Batch token transfers** — Single CPI transfer per direction instead
   of per-bin.

## Running Benchmarks

```bash
# Full benchmark suite
RUSTC_BOOTSTRAP=1 cargo +nightly bench -p dlmm --target-dir /tmp/dlmm-bench

# Full test suite
RUSTC_BOOTSTRAP=1 cargo +nightly test --target-dir /tmp/dlmm-test

# TypeScript property tests
npx mocha --require ts-node/register tests/quote.test.ts
```

