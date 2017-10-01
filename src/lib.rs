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
about 50k different variations of input strings. We make sure that the afsort implementation
sorts in the same way as the standard library's sort_unstable methods.

*/

#[cfg(test)]
extern crate quickcheck;

/// Enhances slices of e.g. Strings to have a `af_sort_unstable` method, as a more idiomatic
/// way to call sort.
///
/// #Example
///
/// ```rust
/// use afsort::AFSortable;
///
/// let mut strings = vec!["c", "a", "b"];
/// strings.af_sort_unstable();
/// assert_eq!(strings, vec!["a", "b", "c"]);
/// ```

pub trait AFSortable {
    #[inline]
    fn af_sort_unstable(&mut self);
}

impl<T> AFSortable for [T]
where
    T: AsRef<[u8]>,
{
    #[inline]
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
/// afsort::sort_unstable(&mut strings);
/// assert_eq!(strings, vec!["a", "b", "c"]);
/// ```
#[inline]
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
/// let mut tuples = vec![("b", 2), ("a", 1)];
///afsort::sort_unstable_by(&mut tuples, |t: &(&str, _) | t.0.as_bytes());
///assert_eq!(tuples, vec![("a", 1), ("b", 2)]);
/// ```
///
/// Footnote: The explicit type annotacion in the closure seems to be needed (even though it should
/// not). See
/// [this discussion](https://users.rust-lang.org/t/lifetime-issue-with-str-in-closure/13137).
#[inline]
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
    {
        //Find min/max to be able to allocate less memory
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
    }
    //No item had a value for this depth
    if min == u16::max_value() {
        return;
    }

    // +2 instead of +1 for special 0 bucket
    let num_items = (max - min + 2) as usize;
    let mut counts: Vec<usize> = vec![0usize; num_items as usize];
    {
        //Count occurences per value. Elements without a value gets
        //the special value 0, while others get the u8 value +1.
        for elem in vec.iter() {
            let radix_val = radix_for_str(to_slice(elem), depth, min);
            counts[radix_val as usize] += 1;
        }
    }


    let mut offsets: Vec<usize> = vec![0usize; num_items as usize];
    {
        //Sets the offsets for each count
        let mut sum = 0usize;
        for i in 0..counts.len() {
            offsets[i as usize] = sum;
            sum += counts[i as usize];
        }
    }

    {
        //Swap objects into the correct bucket, based on the offsets
        let mut next_free = offsets.clone();
        let mut block = 0usize;
        let mut i = 0usize;
        while block < counts.len() - 1 {
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
    }
    {
        //Within each bucket, sort recursively
        for i in 0..offsets.len() - 1 {
            sort_req(
                &mut vec[offsets[i as usize] as usize..offsets[i as usize + 1] as usize],
                to_slice,
                depth + 1,
            );
        }
        sort_req(&mut vec[offsets[offsets.len() - 1]..], to_slice, depth + 1);
    }
}

fn radix_for_str(s: &[u8], d: usize, base: u16) -> u16 {
    if s.len() > d {
        s[d] as u16 + 1 - base
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::AFSortable;
    use quickcheck::QuickCheck;


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
            super::sort_unstable_by(&mut tuples, |t| t.0.as_bytes());
            tuples.into_iter().map(|t| t.0).collect::<Vec<String>>() ==
                copy.into_iter().map(|t| t.0).collect::<Vec<String>>()
        }
        QuickCheck::new().tests(50000).quickcheck(
            compare_sort as
                fn(Vec<(String, u8)>)
                   -> bool,
        );
    }
}
