use std::time::Instant;
use dlmm::math::fixed_point::{q64_mul, q64_div, base_multiplier, inv_base_multiplier, Q64};
use dlmm::math::price_math::{bin_to_price, pow_q64};
use dlmm::math::swap_math::compute_swap_step;

fn bench(label: &str, iterations: u32, f: impl Fn()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    let per_op = elapsed / iterations;
    println!("{:50} {:>8} ns/iter  ({:?} total, {} iters)", label, per_op.as_nanos(), elapsed, iterations);
}

fn main() {
    let iters = 1_000_000u32;
    let step = 10u16;

    println!("=== Fixed-Point Arithmetic ===");
    bench("q64_mul (identity, Q64 × Q64)", iters, || {
        let _ = q64_mul(Q64, Q64).unwrap();
    });
    bench("q64_div (Q64/2 ÷ Q64)", iters, || {
        let _ = q64_div(Q64 / 2, Q64).unwrap();
    });
    bench("base_multiplier (step=10)", iters, || {
        let _ = base_multiplier(step);
    });
    bench("inv_base_multiplier (step=10)", iters, || {
        let _ = inv_base_multiplier(step);
    });

    println!("\n=== Price Computation ===");
    let iters2 = 100_000u32;
    bench("bin_to_price (bin=0)", iters2, || {
        let _ = bin_to_price(0, step).unwrap();
    });
    bench("bin_to_price (bin=100)", iters2, || {
        let _ = bin_to_price(100, step).unwrap();
    });
    bench("bin_to_price (bin=1,000)", iters2, || {
        let _ = bin_to_price(1000, step).unwrap();
    });
    bench("bin_to_price (bin=10,000)", iters2, || {
        let _ = bin_to_price(10000, step).unwrap();
    });
    bench("bin_to_price (bin=-1,000)", iters2, || {
        let _ = bin_to_price(-1000, step).unwrap();
    });

    println!("\n=== Exponentiation ===");
    let base = base_multiplier(step);
    bench("pow_q64 (exp=10)", iters2, || {
        let _ = pow_q64(base, 10).unwrap();
    });
    bench("pow_q64 (exp=100)", iters2, || {
        let _ = pow_q64(base, 100).unwrap();
    });
    bench("pow_q64 (exp=1,000)", iters2, || {
        let _ = pow_q64(base, 1000).unwrap();
    });

    println!("\n=== Swap Step ===");
    bench("swap_step (partial fill, A→B)", iters2, || {
        let _ = compute_swap_step(1000, 5000, 5000, Q64, true, 30).unwrap();
    });
    bench("swap_step (full deplete, A→B)", iters2, || {
        let _ = compute_swap_step(10000, 500, 50, Q64, true, 30).unwrap();
    });
    bench("swap_step (empty bin)", iters2, || {
        let _ = compute_swap_step(1000, 500, 0, Q64, true, 30).unwrap();
    });
}
