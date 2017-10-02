
# American Flag sort for Rust

[![Linux build status](https://api.travis-ci.org/antonha/afsort.png)](https://travis-ci.org/antonha/afsort)
[![Crates.io Version badge](https://img.shields.io/crates/v/afsort.svg)](https://crates.io/crates/afsort)

The afsort crate implements a sorting algorithm based on
[American Flag sort](https://en.wikipedia.org/wiki/American_flag_sort). The implementation is
currently limited to sort byte slices, e.g. Strings. The main motivation is to sort strings of
text, so most of the benchmarks are based on English text strings. When sorting English words, 
this implementation seems to be about 40% faster than `sort_unstable` from the Rust standard 
library.

For small input, this method falls back to the standard library.

# Installation

Add the depndency to your `Cargo.toml`: 

```ignore
[dependencies]
afsort = "0.1"
```
In your crate root:
```ignore
extern crate afsort;
```
**Warning**: Version 0.1.0 is flawed(slow), use 0.1.1.

# Usage

You can now use afsort to e.g. sort arrays of strings or string slices.

```rust
use afsort;
let mut strings = vec!("red", "green", "blue");
afsort::sort_unstable(&mut strings);
assert_eq!(strings, vec!["blue", "green", "red"]);
```

You can also sort by an extractor function, e.g.:

```rust
use afsort;
let mut tuples = vec![("b", 2), ("a", 1)];
afsort::sort_unstable_by(&mut tuples, |t: &(&str, _) | t.0.as_bytes());
assert_eq!(tuples, vec![("a", 1), ("b", 2)]);
```

# Motivation

Essentially, I noticed that sorting of strings took a long time when using the
[fst](https://github.com/BurntSushi/fst) crate, since it requires the input to be ordered.
Since sorting strings is a general problem, this is now a crate.

# Performance

As mentioned, this implementation seems to be about 40% faster than the sort in the standard
library, when sorting strings of random English words.  The implementation is fairly naive,
so I would not be surprised if it could be improved further.

You can run the benchmark tests using `cargo bench` (currently requires nightly rust), like this:

```ignore
% cargo bench
   Compiling afsort v0.1.0 (file:///home/anton/dev/off/afsort)
    Finished release [optimized] target(s) in 5.66 secs
     Running target/release/deps/afsort-ca28db3ba0643253

running 12 tests
test tests::sorts_strings_same_as_unstable ... ignored
test tests::sorts_tuples_same_as_unstable ... ignored
test tests::sort_100000_en_af        ... bench:  20,402,968 ns/iter (+/- 1,512,907)
test tests::sort_100000_en_std       ... bench:  30,132,067 ns/iter (+/- 1,698,223)
test tests::sort_10000_en_af         ... bench:   1,377,303 ns/iter (+/- 114,481)
test tests::sort_10000_en_lower_af   ... bench:   1,371,022 ns/iter (+/- 95,391)
test tests::sort_10000_en_lower_std  ... bench:   2,227,486 ns/iter (+/- 127,281)
test tests::sort_10000_en_sorted_af  ... bench:     878,665 ns/iter (+/- 545,256)
test tests::sort_10000_en_sorted_std ... bench:     618,329 ns/iter (+/- 536,338)
test tests::sort_10000_en_std        ... bench:   2,221,089 ns/iter (+/- 157,461)
test tests::sort_1000_en_af          ... bench:     101,625 ns/iter (+/- 6,946)
test tests::sort_1000_en_std         ... bench:     171,655 ns/iter (+/- 9,844)

test result: ok. 0 passed; 0 failed; 2 ignored; 10 measured; 0 filtered out


```
# Limitations

Currently, this crate only supports sorting elements of `AsRef<[u8]>`. I think that this is a
good first step, since it supports sorting of Strings. There is however no reason why American
Flag sorting should be limited to this data type. Any kind of element that can deliver a radix
to a certain digit/depth can be sorted using this technique.

The American Flag algorithm is unstable, in the same way that sort_unstable in the standard
library. That is, equal elements might be re-ordered.

This crate can _only_ sort strings based on their `utf-8` byte values. For many problems, this
is fine. However, if you want to sort strings for display to a user, Locale might matter. This
crate does not try to address this issue.

# Testing

Testing is done using the [quickcheck](https://github.com/BurntSushi/quickcheck) crate. We run
about 50k different variations of input strings. We treat the standard library's sort_unstable
as the gold standard.

# License 

This repository is licensed under the MIT license, with the exception of the english-american.txt
dictionary file, which comes from the Linux words package, and thus is under GPL. The dictionary 
is only used for testing purposes.
