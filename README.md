# High-Performance Order Book in Rust

Implementation of the `OrderBook` trait for the order book challenge. The goal was to make the hot path operations (`apply_update`, `get_spread`) as fast as possible, ideally under one nanosecond.

All operations measured by the benchmark are now **under 1 ns**.

## Final results

Measured with `cargo run --release` on an Apple M1 Pro (100,000 iterations).

| Operation      | Time     |
|----------------|----------|
| Update (avg)   | 0.86 ns  |
| Update (P99)   | 0.96 ns  |
| Get Best Bid   | 0.43 ns  |
| Get Best Ask   | 0.43 ns  |
| Get Spread     | 0.43 ns  |
| Random Reads   | 0.44 ns  |

## How to run

```bash
# run the tests
cargo test

# run the benchmark
cargo run --release
```

## Project structure

```
src/
├── main.rs          # Entry point, benchmark call and correctness tests
├── interfaces.rs    # OrderBook trait and types (given)
├── orderbook.rs     # My implementation
└── benchmarks.rs    # Benchmark framework (given)
```

## Evolution step by step

I did the project in several steps, each one is a separate commit, to see what each idea brings.

| Step | Commit | Update | Best Bid | Best Ask | Spread | Random Reads |
|------|--------|--------|----------|----------|--------|--------------|
| 1 | Unsorted `Vec` | 147.6 ns | 114.2 ns | 115.3 ns | 231.2 ns | 94.3 ns |
| 2 | More tests | - | - | - | - | - |
| 3 | Sorted `Vec` + binary search | 17.4 ns | 0.43 ns | 0.43 ns | 0.85 ns | 14.8 ns |
| 4 | Array indexed by price + cached best prices | 2.75 ns | 0.43 ns | 0.43 ns | 0.53 ns | 0.44 ns |
| 5 | `inline` and `cold` on the hot path | **0.86 ns** | **0.43 ns** | **0.43 ns** | **0.43 ns** | **0.44 ns** |

### Step 1 - Unsorted Vec

The simplest version: each side is a `Vec<(Price, Quantity)>`. To update, read or find the best price, we go through the whole list. It works but everything is O(n), so it is slow.

### Step 2 - More tests

Before optimizing, I added tests to be sure that the next versions stay correct:

- empty book and only one side filled
- `get_top_levels` (order of the levels, `n` bigger than the number of levels, `n = 0`)
- `get_total_quantity` after updates and removes
- best price after removing the best level
- removing a price that does not exist
- same price on the bid and ask side
- crossed book (negative spread)
- 1000 levels like in the benchmark

### Step 3 - Sorted Vec + binary search

The lists are now kept sorted from the lowest to the highest price:

- the best bid is the last element and the best ask is the first one, so no loop is needed
- to find a price, a binary search is used (O(log n) instead of O(n))
- a new price is inserted at the right place to keep the list sorted

The best prices go under 1 ns, but updates and reads still need a search.

### Step 4 - Array indexed by price

The prices in the benchmark are in a small range, so instead of searching for a price we can use it directly as an index:

```
quantity = bid_quantities[price - base_price]
```

- a quantity of 0 means there is no level at this price
- the arrays grow automatically if a price is outside of the range (`make_room`)
- the best bid and best ask are stored in the struct and updated on each change. When the best level is removed, we look for the next one in the array.

Updates and reads become a simple array access, without any search.

### Step 5 - Faster hot path

By looking at the generated assembly, I saw that `apply_update` was a real function call in the benchmark loop. I made a few small changes, and I tested each one separately to keep only the ones that help:

- `#[inline(always)]` on `apply_update`, so the compiler puts the code directly in the loop
- `#[cold]` on the functions that change the best price. Before, the best price was written to memory on every update and read again just after, so the CPU had to wait. Now it is only written when it really changes, which is rare.
- `#[cold]` on `make_room`, because growing the arrays almost never happens
- only one comparison to check if a price is in the array: if the price is below `base_price`, `(price - base_price) as usize` becomes a huge number, so `i < len` is enough
- `NO_BID` / `NO_ASK` constants instead of `Option<Price>` for the best prices, the comparison is simpler

I also tried release profile settings in `Cargo.toml` (`lto`, `codegen-units = 1`, `panic = "abort"`), but they made no measurable difference, so I did not keep them.

## Limitations

- The array goes from the lowest to the highest price ever seen. If two prices are very far from each other, the array can become very large. It is the usual trade-off of this structure: more memory for more speed. It is fine for the benchmark and the tests.
- `get_total_quantity` goes through the whole array. It is not measured by the benchmark so I kept it simple.
