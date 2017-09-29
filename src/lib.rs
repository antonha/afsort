/*!

The afsort crate implements a sorting implementation based on
[American Flag sort](https://en.wikipedia.org/wiki/American_flag_sort). The implementation is
currently limited to sort byte slices, e.g. Strings. The main motivation is to sort strings of
text, so most of the benchmarks are based on English text strings. This implementation seems
to be about 40% faster than `sort_unstable` from the Rust standard library.

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

# Usage

You can now use afsort to e.g. sort arrays of strings or string slices.

```rust
use afsort;
let mut strings = vec!("red", "green", "blue");
afsort::sort_unstable(strings);
assert_eq!(strings, vec!["blue", "green", "red"]);
```

You can also sort by an extractor function, e.g.:

```rust
use afsort;
let mut tuples = vec![("b", 2), ("a", 1)]
afsort::sort_unstable_by(&mut tuples, |t| t.0.as_bytes());
assert_eq!(strings, vec![("a", 1), ("b", 2)]);
```

# Motivation

Essentially, I noticed that sorting of strings took a long time when using the
[fst](https://github.com/BurntSushi/fst) crate, since it requires the input to be ordered. 
Since sorting strings is a general problem, this is now a crate.

# Performance

As mentioned, this implementation seems to be about 40% faster than the sort in the standard
library, when sorting strings of random English words. That radix-style sorting algorithms can
be faster than quicksort is not anything new. The implementation is fairly naive, so I would
not be surprised if it could be improved further.

You can run the benchmark tests using `cargo bench`, like this:

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
about 50k different variations of input strings. We make sure that the afsort implementation
sorts in the same way as the standard library's sort_unstable methods.

*/

#![feature(test)]
#[cfg(test)]
extern crate test;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
extern crate regex;


/// Base module for the crate. See the crate documentation for more information.
pub mod afsort {

    /// Enhances slices of e.g. Strings to have a `af_sort_unstable` method, as a more idiomatic
    /// way to call sort.
    ///
    /// #Example
    ///
    /// ```rust
    /// let mut strings = vec!["c", "a", "b"];
    /// strings.af_sort_unstable();
    /// assert_eq!(strings, vec!["a", "b", "c"];
    /// ```
    pub trait AFSortable {
        fn af_sort_unstable(&mut self);
    }

    impl<T> AFSortable for [T]
    where
        T: AsRef<[u8]>,
    {
        fn af_sort_unstable(&mut self) {
            sort_unstable(self);
        }
    }


    /// Main sort method.
    ///
    /// #Example 
    ///
    /// ```rust
    /// let mut strings = vec!["c", "a", "b"];
    /// strings.af_sort_unstable();
    /// assert_eq!(strings, vec!["a", "b", "c"];
    /// ```
    pub fn sort_unstable<'a, T>(vec: &'a mut [T])
    where
        T: AsRef<[u8]>,
    {
        sort_unstable_by(vec, (|s| s.as_ref()));
    }

    /// Sort method which accepts function to convert elements to &[u8].
    ///
    /// #Example
    ///
    /// ```rust
    /// let mut tuples = vec![("b", 2), ("a", 1)]
    ///afsort::sort_unstable_by(&mut tuples, |t| t.0.as_bytes());
    ///assert_eq!(strings, vec![("a", 1), ("b", 2)]);
    /// ```
    pub fn sort_unstable_by<T, F>(vec: &mut [T], to_slice: F)
    where
        F: Fn(&T) -> &[u8],
    {
        sort_req(vec, &to_slice, 0);
    }

    fn sort_req<T, F>(vec: &mut [T], to_slice: &F, depth: usize)
    where
        F: Fn(&T) -> &[u8],
    {
        if vec.len() <= 32 {
            vec.sort_unstable_by(|e1, e2| to_slice(e1).cmp(to_slice(e2)));
            return;
        }
        let mut min = u16::max_value();
        let mut max = 0u16;
        for elem in vec.iter() {
            let val = to_slice(elem);
            if val.len() > depth {
                let radix_val = val[depth] as u16;
                if radix_val < min {
                    min = radix_val;
                }
                if radix_val > max {
                    max = radix_val;
                }
            }
        }
        if min == u16::max_value() {
            return;
        }

        let num_items = (max - min + 2) as usize;
        let mut counts: Vec<usize> = vec![0usize; num_items as usize];

        for elem in vec.iter() {
            let radix_val = radix_for_str(to_slice(elem), depth, min);
            counts[radix_val as usize] += 1;
        }

        let mut sum = 0usize;
        let mut offsets: Vec<usize> = vec![0usize; num_items as usize];
        for i in 0..num_items {
            offsets[i as usize] = sum;
            sum += counts[i as usize];
        }
        let mut next_free = offsets.clone();

        let mut block = 0usize;
        let mut i = 0usize;
        while block < num_items - 1 {
            if i >= offsets[block as usize + 1] as usize {
                block += 1;
            } else {
                let radix_val = radix_for_str(to_slice(&vec[i]), depth, min);
                if radix_val == block as u16 {
                    i += 1;
                } else {
                    vec.swap(i as usize, next_free[radix_val as usize] as usize);
                    next_free[radix_val as usize] += 1;
                }
            }
        }
        for i in 0..num_items - 1 {
            sort_req(
                &mut vec[offsets[i as usize] as usize..offsets[i as usize + 1] as usize],
                to_slice,
                depth + 1,
            );
        }
        sort_req(&mut vec[offsets[num_items - 1]..], to_slice, depth + 1);
    }

    fn radix_for_str(s: &[u8], d: usize, base: u16) -> u16 {
        if s.len() > d {
            s[d] as u16 + 1 - base
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {

    use afsort;
    use afsort::AFSortable;
    use quickcheck::QuickCheck;
    use test::Bencher;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use rand::{self, Rng};
    use regex::Regex;
    use std::path::PathBuf;

    #[test]
    fn sorts_strings_same_as_unstable() {
        fn compare_sort(mut strings: Vec<String>) -> bool {
            let mut copy = strings.clone();
            copy.sort_unstable();
            strings.af_sort_unstable();
            strings == copy
        }
        QuickCheck::new().tests(50000).quickcheck(
            compare_sort as
                fn(Vec<String>) -> bool,
        );
    }

    #[test]
    fn sorts_tuples_same_as_unstable() {
        fn compare_sort(mut tuples: Vec<(String, u8)>) -> bool {
            let mut copy = tuples.clone();
            copy.sort_unstable_by(|t1, t2| t1.0.cmp(&t2.0));
            afsort::sort_unstable_by(&mut tuples, |t| t.0.as_bytes());
            tuples.into_iter().map(|t| t.0).collect::<Vec<String>>() ==
                copy.into_iter().map(|t| t.0).collect::<Vec<String>>()
        }
        QuickCheck::new().tests(50000).quickcheck(
            compare_sort as
                fn(Vec<(String, u8)>)
                   -> bool,
        );
    }

    #[bench]
    fn sort_1000_en_std(b: &mut Bencher) {
        let strings = strings_en(&Regex::new(r".*").unwrap(), 1_000);
        b.iter(|| strings.clone().sort_unstable())
    }

    #[bench]
    fn sort_1000_en_af(b: &mut Bencher) {
        let strings = strings_en(&Regex::new(r".*").unwrap(), 1_000);
        b.iter(|| strings.clone().af_sort_unstable())
    }

    #[bench]
    fn sort_10000_en_std(b: &mut Bencher) {
        let strings = strings_en(&Regex::new(r".*").unwrap(), 10_000);
        b.iter(|| strings.clone().sort_unstable())
    }

    #[bench]
    fn sort_10000_en_af(b: &mut Bencher) {
        let strings = strings_en(&Regex::new(r".*").unwrap(), 10_000);
        b.iter(|| strings.clone().af_sort_unstable())
    }

    #[bench]
    fn sort_100000_en_std(b: &mut Bencher) {
        let strings = strings_en(&Regex::new(r".*").unwrap(), 100_000);
        b.iter(|| strings.clone().sort_unstable())
    }

    #[bench]
    fn sort_100000_en_af(b: &mut Bencher) {
        let strings = strings_en(&Regex::new(r".*").unwrap(), 100_000);
        b.iter(|| strings.clone().af_sort_unstable())
    }

    #[bench]
    fn sort_10000_en_sorted_std(b: &mut Bencher) {
        let mut strings = strings_en(&Regex::new(r".*").unwrap(), 10_000);
        strings.sort_unstable();
        b.iter(|| strings.clone().sort_unstable())
    }

    #[bench]
    fn sort_10000_en_sorted_af(b: &mut Bencher) {
        let mut strings = strings_en(&Regex::new(r".*").unwrap(), 10_000);
        strings.sort_unstable();
        b.iter(|| strings.clone().af_sort_unstable())
    }

    #[bench]
    fn sort_10000_en_lower_std(b: &mut Bencher) {
        let strings = strings_en(&Regex::new(r"^[a-z]+$").unwrap(), 10000);
        b.iter(|| strings.clone().sort_unstable())
    }

    #[bench]
    fn sort_10000_en_lower_af(b: &mut Bencher) {
        let strings = strings_en(&Regex::new(r"^[a-z]+$").unwrap(), 10000);
        b.iter(|| strings.clone().af_sort_unstable())
    }

    fn strings_en(re: &Regex, n: usize) -> Vec<String> {
        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let f = File::open(d.join("test_resources/american-english.txt")).unwrap();
        let b = BufReader::new(f);
        let mut strings = b.lines()
            .map(|l| l.unwrap())
            .filter(|l| re.is_match(l))
            .collect::<Vec<String>>();
        let mut rng = rand::thread_rng();
        rng.shuffle(&mut strings);
        strings.into_iter().take(n).collect::<Vec<String>>()
    }


}
