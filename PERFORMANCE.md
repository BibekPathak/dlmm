# DLMM Performance

## Account Sizes

| Account | Size | Zero-Copy | Notes |
|---------|------|-----------|-------|
| Pool | 201 B | No | Single account per pool pair; read on every instruction |
| Position | 145 B | No | One per user per pool; read on add/remove/collect |
| BinArray | 2096 B | **Yes** | One per 64-bin contiguous range; read/written every swap |
| Bin (within BinArray) | 32 B | **Yes** | 4 × u64; zero-copy access via bytemuck |

### Account size breakdown

```
Pool:    8 (discriminator) + 32+32+32+32+32+2+2+2+4+4+8+8+2+2+16+8+8+8+1+7 = 201 B
Position: 8 (discriminator) + 32+32+4+4+8+8+8+8+8+8+8+1 = 145 B
BinArray: 8 (discriminator) + 32+4+1+3+32*64 = 2096 B  (~2 KB)
```

The maximum transaction size on Solana is 1232 bytes for a single packet
(though larger transactions can be sent via versioned transactions with
address lookup tables). BinArray at 2 KB fits comfortably.

## Zero-Copy Performance

BinArray uses `#[account(zero_copy)]` with `#[repr(C)]` layout, providing:

- **O(1) random access** to any bin via constant-time indexing
- **No Borsh serialization/deserialization overhead** on reads/writes
- **Direct memory mapping** through `bytemuck::from_bytes_mut`

Compare with standard `#[account]`: Borsh deserialization on every read
would add ~200-500 CU per account access. With 64 bins and potentially
multiple arrays traversed per swap, zero-copy saves thousands of CUs.

## Price Computation: Before vs After Optimization

### Before (Phase 8 — initial implementation)

Each bin iteration called `bin_to_price(bin_id)` which runs **O(log n)**
binary exponentiation. For a swap crossing `K` bins:
- Total: `K * O(log n)` multiplications
- Worst case: 1000 bins × ~17 multiplications = ~17,000 mul operations

### After (Phase 12 — incremental price)

Price is computed once with `bin_to_price(active_bin_id)`, then updated
per-bin with a single `q64_mul(price, step_multiplier)`:
- Initial: 1 × `bin_to_price` = O(log n) multiplications
- Per bin: 1 × `q64_mul` = O(1) multiplication
- Total: `O(log n) + K` multiplications
- Worst case: ~17 + 1000 = ~1,017 mul operations (~17× improvement)

## Compute Unit Estimates

All estimates assume worst-case scenario with max bins traversed.

### Initialize Pool

| Operation | Est. CUs |
|-----------|---------|
| Account creation (Pool, 2 vaults) | ~5,000 |
| Pool field writes (15+ fields) | ~500 |
| Token mint validation | ~1,000 |
| **Total** | **~6,500** |

### Initialize BinArray

| Operation | Est. CUs |
|-----------|---------|
| Account creation (2 KB) | ~3,000 |
| Zero-fill 64 bins | ~1,000 |
| Pool key + start_bin_id writes | ~100 |
| **Total** | **~4,100** |

### Add/Remove Liquidity (10 deposits)

| Operation | Est. CUs |
|-----------|---------|
| BinArray data access (zero-copy) | ~500 |
| Bin amount updates (10 × 2 u64) | ~200 |
| Price validation (10 × bin_to_price) | ~1,700 |
| SPL token CPI transfer | ~5,000 |
| Position tracking | ~200 |
| **Total** | **~7,600** |

### Swap (10 bins traversed, ExactIn)

| Operation | Est. CUs |
|-----------|---------|
| BinArray data access (zero-copy, 1-2 arrays) | ~500 |
| Incremental price computation (10 × q64_mul) | ~200 |
| Swap step computation (10 × compute_swap_step) | ~1,000 |
| Bin amount updates (10 × u64) | ~200 |
| SPL token CPI transfers (2 ×) | ~10,000 |
| Volatility update (bin_to_price + decay) | ~200 |
| Event emission | ~500 |
| **Total** | **~12,600** |

### Worst-Case Swap (200 bins)

| Operation | Est. CUs |
|-----------|---------|
| BinArray traversal (4 arrays × 50 bins) | ~20,000 |
| Price computation (incremental, 200 × q64_mul) | ~4,000 |
| Swap step computation (200 × compute_swap_step) | ~20,000 |
| Bin amount updates (200 × 2 u64) | ~4,000 |
| SPL token CPI transfers (2 ×) | ~10,000 |
| Volatility + event | ~1,000 |
| **Total** | **~59,000** |

Solana's compute budget default is **200,000 CU per transaction**.
The worst-case swap uses ~59,000 CU (~30% of budget), leaving
~141,000 CU for transaction overhead and user validation.

A swap crossing **~600 bins** would hit the compute limit.

## Gas Optimization Techniques Used

1. **Zero-copy accounts** — BinArray uses `#[account(zero_copy)]`
   to avoid Borsh serialization/deserialization.

2. **Incremental price computation** — O(log n) first price, then
   O(1) per-bin using `q64_mul(price, step_multiplier)`.

3. **Per-BinArray iteration** — The outer loop processes one BinArray
   at a time, avoiding repeated PDA lookups for bin → array mapping.

4. **bytemuck direct access** — Casting account data directly to
   `&mut BinArray` instead of using `AccountLoader::load_mut()`,
   which avoids Anchor's internal deserialization checks.

5. **Checked math with early exit** — `checked_add`/`checked_sub`
   with immediate error returns prevent wasted computation on
   overflow paths.

6. **Batch token transfers** — Single CPI transfer per token
   direction (two total: one in, one out) rather than per-bin.

## Benchmarking (when deployed)

To run benchmarks on a local validator:

```bash
anchor test --skip-deploy  # uses already-deployed program
```

Or measure compute units with `solana logs`:

```bash
solana-test-validator --log
# In another terminal:
anchor test 2>&1 | grep -E "Program return|Compute Budget"
```

Key metrics to measure:
- CU per instruction type
- CU per bin traversed
- CU per liquidity deposit
- Maximum liquidity deposits before CU limit
- Maximum swap amount before CU limit
