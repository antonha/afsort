# American Flag sort for Rust

[![Linux build status](https://travis-ci.org/antonha/afsort.svg?branch=master)](https://travis-ci.org/antonha/afsort)
[![Crates.io Version badge](https://img.shields.io/crates/v/afsort.svg)](https://crates.io/crates/afsort)

The afsort crate implements a sorting algorithm based on
[American Flag sort](https://en.wikipedia.org/wiki/American_flag_sort). The implementation is
currently limited to sort byte slices, e.g. Strings. The main motivation is to sort strings of
text, so most of the benchmarks are based on English text strings. When sorting English words,
this implementation seems to be about 40% faster than `sort_unstable` from the Rust standard
library.

For small input, this method falls back to the standard library.

# Installation

Add the dependency to your `Cargo.toml`:

```ignore
[dependencies]
afsort = "0.2"
```
In your crate root:
```ignore
extern crate afsort;
```

**Warning**: Version 0.1.0 is flawed(slow), use 0.1.1.

**Note on upgrading 0.1.x -> 0.2.x**: The method `afsort::sort_unstable(&mut [AsRef<u8>])` has
been removed. Use the af_sort_unstable from the `AFSortable` trait instead.

# Usage

You can now afsort to e.g. sort arrays of strings or string slices.

```rust
use afsort::AFSortable;
let mut strings = vec!("red", "green", "blue");
strings.af_sort_unstable();
assert_eq!(strings, vec!["blue", "green", "red"]);
```

It also works on u8, u16, u32 and u64:

```rust
use afsort::AFSortable;
let mut strings = vec!(1u32, 2u32, 7u32);
strings.af_sort_unstable();
assert_eq!(strings, vec![1u32, 2u32, 7u32]);
```

You can also sort by an extractor function, e.g.:

```rust
use afsort;
let mut tuples = vec![("b", 2), ("a", 1)];
afsort::sort_unstable_by(&mut tuples, |t| &t.0);
assert_eq!(tuples, vec![("a", 1), ("b", 2)]);
```

The `af_sort_unstable()` method is implemented for all slices of values that implement the
`afsort::DigitAt` and the `Ord` traits. The `DigitAt` trait is implemented for `&str`
, `String`, `[u8]`, `u8`, `u16`, `u32` and `u64`. All of these also implement Ord. You can also
implement this trait for any other type.

# Motivation

Essentially, I noticed that sorting of strings took a long time when using the
[fst](https://github.com/BurntSushi/fst) crate, since it requires the input to be ordered.
Since sorting strings is a general problem, this is now a crate.

# Performance

As mentioned, this implementation seems to be about 40% faster than the sort in the standard
library, when sorting strings of random English words.  It is slower for strings that are
already sorted. The implementation is fairly naive, so I would not be surprised if it could
be improved further.

For numbers, it currently seems to be slower than the standard library. I suspect this is due
to more swaps happening in afsort than in the standard library. I want to fix this.

This will be heavily affected by the distribution of values in the input though. As always with
performance: _your milage may vary_. Profile your usage.

You can run the benchmark tests using `cargo bench` (currently requires nightly rust), like this:

```ignore
% cargo bench
    Finished release [optimized] target(s) in 0.0 secs
     Running target/release/deps/afsort-2f0d4e495216be99
running 10 tests
test tests::correct_radix_for_u16 ... ignored
test tests::correct_radix_for_u32 ... ignored
test tests::correct_radix_for_u64 ... ignored
test tests::correct_radix_for_u8 ... ignored
test tests::sorts_strings_same_as_unstable ... ignored
test tests::sorts_tuples_same_as_unstable ... ignored
test tests::sorts_u16_same_as_unstable ... ignored
test tests::sorts_u32_same_as_unstable ... ignored
test tests::sorts_u64_same_as_unstable ... ignored
test tests::sorts_u8_same_as_unstable ... ignored
test result: ok. 0 passed; 0 failed; 10 ignored; 0 measured; 0 filtered out
     Running target/release/deps/bench-42a0c77149fb906a
running 16 tests
test sort_en_strings_lower_10_000_af   ... bench:   1,881,300 ns/iter (+/- 618,858)
test sort_en_strings_lower_10_000_std  ... bench:   2,594,388 ns/iter (+/- 767,774)
test sort_en_strings_rand_100_000_af   ... bench:  23,101,465 ns/iter (+/- 12,052,025)
test sort_en_strings_rand_100_000_std  ... bench:  31,536,516 ns/iter (+/- 12,910,887)
test sort_en_strings_rand_10_000_af    ... bench:   1,588,372 ns/iter (+/- 568,509)
test sort_en_strings_rand_10_000_std   ... bench:   2,193,132 ns/iter (+/- 648,297)
test sort_en_strings_sorted_10_000_af  ... bench:     806,419 ns/iter (+/- 128,186)
test sort_en_strings_sorted_10_000_std ... bench:     589,161 ns/iter (+/- 340,707)
test sort_u16_1_000_000_af             ... bench:  19,442,855 ns/iter (+/- 1,642,992)
test sort_u16_1_000_000_std            ... bench:  21,401,736 ns/iter (+/- 3,607,120)
test sort_u32_1_000_000_af             ... bench:  31,682,863 ns/iter (+/- 5,254,810)
test sort_u32_1_000_000_std            ... bench:  30,809,651 ns/iter (+/- 1,623,271)
test sort_u64_1_000_000_af             ... bench:  39,730,940 ns/iter (+/- 6,139,556)
test sort_u64_1_000_000_std            ... bench:  32,477,660 ns/iter (+/- 1,733,969)
test sort_u8_1_000_af                  ... bench:      11,330 ns/iter (+/- 702)
test sort_u8_1_000_std                 ... bench:       8,764 ns/iter (+/- 163)
test result: ok. 0 passed; 0 failed; 0 ignored; 16 measured; 0 filtered out
```
# Limitations

The American Flag algorithm is unstable, in the same way that sort_unstable in the standard
library. That is, equal elements might be re-ordered.

This crate can _only_ sort strings based on their `utf-8` byte values. For many problems, this
is fine. However, if you want to sort strings for display to a user, Locale might matter. This
crate does not try to address this issue.

# Testing

Testing is done using the [quickcheck](https://github.com/BurntSushi/quickcheck) crate. We run
about 50k different variations of input strings & numbers. We treat the standard library's
sort_unstable as the gold standard. This means that it is very likely that this library is as
bug-free (at least in a functional sense) as the standard library.

# License 

This repository is licensed under the MIT license, with the exception of the english-american.txt
dictionary file, which comes from the Linux words package, and thus is under GPL. The dictionary 
is only used for testing purposes.
