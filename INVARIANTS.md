# DLMM Formal Invariants

This document enumerates the invariants that the DLMM protocol guarantees
at all times. Violation of any invariant constitutes a bug.

---

## 1. Pool Invariants

| # | Invariant | Enforced By | Notes |
|---|-----------|-------------|-------|
| P1 | `active_bin_id ∈ [-BIN_ID_RANGE, BIN_ID_RANGE]` | `initialize_pool` validation + swap handler bounds | Set once on creation, updated by swaps |
| P2 | `bin_step_bps ∈ [1, 10_000]` | `initialize_pool` require check | Zero step would make all bins same price |
| P3 | `base_fee_bps ≤ 10_000` | `initialize_pool` require check | Fee cannot exceed 100% |
| P4 | `protocol_fee_bps ≤ base_fee_bps` | `initialize_pool` require check | Protocol share cannot exceed total fee |
| P5 | `pending_protocol_fees_x/y` never negative | `checked_add` in swap handler | Balance monotonically increases |
| P6 | `variable_fee_bps ≤ 200` | `update_volatility` caps at 200 bps | Volatility component bounded |
| P7 | `vol_accumulator ≤ 100_000` | `calculate_variable_fee` clamps at 100K | Capped volatility accumulator |

## 2. BinArray Invariants

| # | Invariant | Enforced By | Notes |
|---|-----------|-------------|-------|
| B1 | `∀ bin ∈ BinArray: bin.amount_x ≥ 0 ∧ bin.amount_y ≥ 0` | All `checked_sub` return `InsufficientLiquidity` error | Never negative |
| B2 | `∀ bin: bin.fee_x ≥ 0 ∧ bin.fee_y ≥ 0` | `checked_add` only | Fees only accumulate |
| B3 | `start_bin_id % 64 == 0` | `initialize_bin_array` require check | Aligned to BinArray boundary |
| B4 | `BinArray.pool == Pool that created it` | PDA seed derivation | Immutable after creation |
| B5 | ∀ bin: `deductions ≤ current balance` | `remove_liquidity` + `swap` check | Every withdrawal checks sufficient balance |

## 3. Position Invariants

| # | Invariant | Enforced By | Notes |
|---|-----------|-------------|-------|
| Q1 | `lower_bin_id ≤ upper_bin_id` | `open_position` require check | Valid bin range |
| Q2 | `position.owner == signer` for all modifying ops | Anchor `has_one = owner` constraint | Only owner can modify |
| Q3 | `position.pool == Pool` being operated on | Anchor `has_one = pool` constraint | Position bound to one pool |
| Q4 | `total_liquidity_x/y` never negative | `checked_sub` in add/remove handlers | Underflow protection |
| Q5 | `fees_owed_x/y` never decrease | Only `checked_add` in `collect_fees` | Monotonic until collection |
| Q6 | `fees_owed_x = sum_over_bins(position_share_of_bin_fee_x)` | `collect_fees` proportional calculation | Fee distribution is linear |
| Q7 | `deposit.bin_id ∈ [lower, upper]` | `add_liquidity` range check | No deposits outside position range |

## 4. Token Balance Invariants

| # | Invariant | Enforced By | Notes |
|---|-----------|-------------|-------|
| T1 | `vault_a_balance == Σ(bin.amount_x) + pending_protocol_fees_x + Σ(bin.fee_x)` | Cumulative accounting | All tokens in vault are accounted for in bins + fees |
| T2 | `vault_b_balance == Σ(bin.amount_y) + pending_protocol_fees_y + Σ(bin.fee_y)` | Cumulative accounting | Symmetric for token B |
| T3 | `user_balance_change == amount_in - amount_out - fees` | CPI token transfers | User sees correct net change |
| T4 | No token creation or destruction | All transfers are CPI from existing accounts | Every token movement is a transfer between two existing accounts |

## 5. Swap Invariants

| # | Invariant | Enforced By | Notes |
|---|-----------|-------------|-------|
| S1 | `amount_in ≥ min_amount_out` (ExactIn) | Post-swap `require` check | Slippage protection |
| S2 | `total_in ≤ max_amount_in` (ExactOut) | Post-swap `require` check | Maximum spend cap |
| S3 | `actual_fee ≤ amount_in × fee_bps / 10000` | `apply_bps` truncation | Fee capped by rate |
| S4 | Active bin ID only moves in the swap direction | Direction parameter (1 or -1) | No reverse movement |
| S5 | Bins are traversed monotonically | Loop structure in swap handler | Sequential bin traversal |
| S6 | `∀ bin: amount_change ≤ pre_swap_balance` | `compute_swap_step` checks available | Cannot overdraft a bin |

## 6. Architectural Invariants

| # | Invariant | Enforced By | Notes |
|---|-----------|-------------|-------|
| A1 | Pool PDA is derived from `[b"pool", mint_a, mint_b]` | Anchor PDA constraint | Unique per token pair |
| A2 | BinArray PDA is derived from `[b"bin_array", pool, start_bin_id]` | Anchor PDA constraint | Unique per pool × bin range |
| A3 | Position PDA is derived from `[b"position", pool, owner]` | Anchor PDA constraint | Unique per user per pool |
| A4 | Pool vaults are owned by pool PDA | Token account authority | Only program can withdraw from vaults |
| A5 | All math operations use `checked_*` or `saturating_*` | Code audit | No silent overflow |

## 7. Invariant Cross-References

```
 AddLiquidity:  Q7, Q2, Q3, Q4, T3, A3
 RemoveLiquidity: Q2, Q3, Q4, B1, B5, T3, A1, A4
 Swap:          S1-S6, P1, P5-P7, B1, B5, T1-T4
 CollectFees:   Q2, Q3, Q5, Q6, T3, A4
 InitializePool: P1-P4, A1, A4
 InitializeBinArray: B3, B4, A2
 OpenPosition:  Q1, Q2, Q3, A3
```
