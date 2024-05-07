/*!

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
afsort = "0.1"
```
In your crate root:
```ignore
extern crate afsort;
```

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

*/

#[cfg(test)]
extern crate quickcheck;

use std::borrow::Cow;

/// Specifies that a type can deliver a radix at a certain digit/depth.
pub trait DigitAt {
    /// Extracts a radix value at a certain digit for a type. Should return None if no value exists
    /// at the digit.
    ///
    /// #Example
    ///
    /// ```rust
    /// use afsort::DigitAt;
    ///
    /// let num = 0x0502u16;
    /// assert_eq!(Some(5), num.get_digit_at(0));
    /// assert_eq!(Some(2), num.get_digit_at(1));
    /// assert_eq!(None, num.get_digit_at(2));
    /// ```
    fn get_digit_at(&self, digit: usize) -> Option<u8>;
}

impl DigitAt for u8 {
    #[inline]
    fn get_digit_at(&self, digit: usize) -> Option<u8> {
        if digit == 0 {
            Some(*self)
        } else {
            None
        }
    }
}

impl DigitAt for u16 {
    #[inline]
    fn get_digit_at(&self, digit: usize) -> Option<u8> {
        match digit {
            0 => Some(((self & 0xFF00) >> 8) as u8),
            1 => Some((self & 0xFF) as u8),
            _ => None,
        }
    }
}

impl DigitAt for u32 {
    #[inline]
    fn get_digit_at(&self, digit: usize) -> Option<u8> {
        match digit {
            0 => Some(((self & 0xFF000000) >> 24) as u8),
            1 => Some(((self & 0xFF0000) >> 16) as u8),
            2 => Some(((self & 0xFF00) >> 8) as u8),
            3 => Some((*self & 0xFF) as u8),
            _ => None,
        }
    }
}

impl DigitAt for u64 {
    #[inline]
    fn get_digit_at(&self, digit: usize) -> Option<u8> {
        match digit {
            0 => Some(((self & 0xFF00000000000000) >> 56) as u8),
            1 => Some(((self & 0xFF000000000000) >> 48) as u8),
            2 => Some(((self & 0xFF0000000000) >> 40) as u8),
            3 => Some(((self & 0xFF00000000) >> 32) as u8),
            4 => Some(((self & 0xFF000000) >> 24) as u8),
            5 => Some(((self & 0xFF0000) >> 16) as u8),
            6 => Some(((self & 0xFF00) >> 8) as u8),
            7 => Some((self & 0xFF) as u8),
            _ => None,
        }
    }
}

impl<'a> DigitAt for &'a str {
    #[inline]
    fn get_digit_at(&self, digit: usize) -> Option<u8> {
        if self.len() > digit {
            Some(self.as_bytes()[digit])
        } else {
            None
        }
    }
}

impl DigitAt for String {
    #[inline]
    fn get_digit_at(&self, digit: usize) -> Option<u8> {
        if self.len() > digit {
            Some(self.as_bytes()[digit])
        } else {
            None
        }
    }
}

impl DigitAt for [u8] {
    #[inline]
    fn get_digit_at(&self, digit: usize) -> Option<u8> {
        if self.len() > digit {
            Some(self[digit])
        } else {
            None
        }
    }
}

impl<'a> DigitAt for &'a [u8] {
    #[inline]
    fn get_digit_at(&self, digit: usize) -> Option<u8> {
        if self.len() > digit {
            Some(self[digit])
        } else {
            None
        }
    }
}

impl<'a> DigitAt for Cow<'a, str> {
    #[inline]
    fn get_digit_at(&self, digit: usize) -> Option<u8> {
        if self.len() > digit {
            Some(self.as_bytes()[digit])
        } else {
            None
        }
    }
}

impl<T: AsRef<dyn DigitAt>> DigitAt for T {
    #[inline]
    fn get_digit_at(&self, digit: usize) -> Option<u8> {
        self.as_ref().get_digit_at(digit)
    }
}

/// Enhances slices of `DigitAt` implementors to have a `af_sort_unstable` method.
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
    fn af_sort_unstable(&mut self);
}

impl<T> AFSortable for [T]
where
    T: DigitAt + Ord,
{
    #[inline]
    fn af_sort_unstable(&mut self) {
        sort_unstable_by(self, ident);
    }
}

#[inline]
fn ident<T>(t: &T) -> &T {
    t
}

/// Sort method which accepts function to convert elements to &[u8].
///
/// #Example
///
/// ```rust
/// let mut tuples = vec![("b", 2), ("a", 1)];
///afsort::sort_unstable_by(&mut tuples, |t| &t.0);
///assert_eq!(tuples, vec![("a", 1), ("b", 2)]);
/// ```
///
/// Footnote: The explicit type annotacion in the closure seems to be needed (even though it should
/// not). See
/// [this discussion](https://users.rust-lang.org/t/lifetime-issue-with-str-in-closure/13137).
#[inline]
pub fn sort_unstable_by<T, O, S>(vec: &mut [T], sort_by: S)
where
    O: Ord + DigitAt + ?Sized,
    S: Fn(&T) -> &O,
{
    sort_req(
        vec,
        &|item, digit| sort_by(item).get_digit_at(digit),
        &|remaining| remaining.sort_unstable_by(|e1, e2| sort_by(e1).cmp(sort_by(e2))),
        0,
    );
}

/// Like [sort_unstable_by] except it can be used to sort an arbitrary slice without needing to conform to DigitAt
/// and using whatever additional sorting algorithm you'd like (e.g. glidesort).
#[inline]
pub fn sort_unstable_by_digit<T, S, C>(vec: &mut [T], by_digit: S, sort_remaining: C)
where
    S: Fn(&T, usize) -> Option<u8>,
    C: Fn(&mut [T]),
{
    sort_req(vec, &by_digit, &sort_remaining, 0);
}

fn sort_req<T, S, C>(vec: &mut [T], by_digit: &S, sort_remaining: &C, depth: usize)
where
    S: Fn(&T, usize) -> Option<u8>,
    C: Fn(&mut [T]),
{
    if vec.len() <= 32 {
        sort_remaining(vec);
        return;
    }
    let mut min = u16::max_value();
    let mut max = 0u16;
    {
        //Find min/max to be able to allocate less memory
        for elem in vec.iter() {
            if let Some(v) = by_digit(elem, depth) {
                let radix_val = v as u16;
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
    let mut counts: Vec<usize> = vec![0usize; num_items];
    {
        //Count occurences per value. Elements without a value gets
        //the special value 0, while others get the u8 value +1.
        for elem in vec.iter() {
            let radix_val = match by_digit(elem, depth) {
                Some(r) => r as u16 + 1 - min,
                None => 0,
            };
            counts[radix_val as usize] += 1;
        }
    }

    let mut offsets: Vec<usize> = vec![0usize; num_items];
    {
        //Sets the offsets for each count
        let mut sum = 0usize;
        for i in 0..counts.len() {
            offsets[i] = sum;
            sum += counts[i];
        }
    }
    {
        //Swap objects into the correct bucket, based on the offsets
        let mut next_free = offsets.clone();
        let mut block = 0usize;
        let mut i = 0usize;
        while block < counts.len() - 1 {
            if i >= offsets[block + 1] as usize {
                block += 1;
            } else {
                let radix_val = match by_digit(&vec[i], depth) {
                    Some(r) => r as u16 + 1 - min,
                    None => 0,
                };
                if radix_val == block as u16 {
                    i += 1;
                } else {
                    vec.swap(i, next_free[radix_val as usize] as usize);
                    next_free[radix_val as usize] += 1;
                }
            }
        }
    }
    {
        //Within each bucket, sort recursively. We can skip the first, since all elements
        //in it have no radix at this depth, and thus are equal.
        for i in 1..offsets.len() - 1 {
            sort_req(
                &mut vec[offsets[i]..offsets[i + 1]],
                by_digit,
                sort_remaining,
                depth + 1,
            );
        }
        sort_req(
            &mut vec[offsets[offsets.len() - 1]..],
            by_digit,
            sort_remaining,
            depth + 1,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::AFSortable;
    use super::DigitAt;
    use quickcheck::QuickCheck;
    use std::borrow::Cow;

    #[test]
    fn sorts_strings_same_as_unstable() {
        fn compare_sort(mut strings: Vec<String>) -> bool {
            let mut copy = strings.clone();
            copy.sort_unstable();
            strings.af_sort_unstable();
            strings == copy
        }
        QuickCheck::new()
            .tests(50000)
            .quickcheck(compare_sort as fn(Vec<String>) -> bool);
    }

    #[test]
    fn sorts_cow_str_same_as_unstable() {
        fn compare_sort(strings: Vec<String>) -> bool {
            let mut cows: Vec<Cow<str>> = strings.into_iter().map(Cow::Owned).collect();
            let mut copy = cows.clone();
            copy.sort_unstable();
            cows.af_sort_unstable();
            cows == copy
        }
        QuickCheck::new()
            .tests(50000)
            .quickcheck(compare_sort as fn(Vec<String>) -> bool);
    }

    #[test]
    fn sorts_u8_ref_same_as_unstable() {
        fn compare_sort(nums: Vec<Vec<u8>>) -> bool {
            let mut refs: Vec<&[u8]> = nums.iter().map(|i| i.as_slice()).collect();
            let mut copy = refs.clone();
            copy.sort_unstable();
            refs.af_sort_unstable();
            refs == copy
        }
        QuickCheck::new()
            .tests(50000)
            .quickcheck(compare_sort as fn(Vec<Vec<u8>>) -> bool);
    }

    #[test]
    fn sorts_u8_same_as_unstable() {
        fn compare_sort(mut nums: Vec<u8>) -> bool {
            let mut copy = nums.clone();
            copy.sort_unstable();
            nums.af_sort_unstable();
            nums == copy
        }
        QuickCheck::new()
            .tests(50000)
            .quickcheck(compare_sort as fn(Vec<u8>) -> bool);
    }

    #[test]
    fn sorts_u16_same_as_unstable() {
        fn compare_sort(mut nums: Vec<u16>) -> bool {
            let mut copy = nums.clone();
            copy.sort_unstable();
            nums.af_sort_unstable();
            nums == copy
        }
        QuickCheck::new()
            .tests(50000)
            .quickcheck(compare_sort as fn(Vec<u16>) -> bool);
    }

    #[test]
    fn sorts_u32_same_as_unstable() {
        fn compare_sort(mut nums: Vec<u32>) -> bool {
            let mut copy = nums.clone();
            copy.sort_unstable();
            nums.af_sort_unstable();
            nums == copy
        }
        QuickCheck::new()
            .tests(50000)
            .quickcheck(compare_sort as fn(Vec<u32>) -> bool);
    }

    #[test]
    fn sorts_u64_same_as_unstable() {
        fn compare_sort(mut nums: Vec<u64>) -> bool {
            let mut copy = nums.clone();
            copy.sort_unstable();
            nums.af_sort_unstable();
            nums == copy
        }
        QuickCheck::new()
            .tests(50000)
            .quickcheck(compare_sort as fn(Vec<u64>) -> bool);
    }

    #[test]
    fn sorts_tuples_same_as_unstable() {
        fn compare_sort(mut tuples: Vec<(String, u8)>) -> bool {
            let mut copy = tuples.clone();
            copy.sort_unstable_by(|t1, t2| t1.0.cmp(&t2.0));
            super::sort_unstable_by(&mut tuples, |t| &t.0);
            tuples.into_iter().map(|t| t.0).collect::<Vec<String>>()
                == copy.into_iter().map(|t| t.0).collect::<Vec<String>>()
        }
        QuickCheck::new()
            .tests(50000)
            .quickcheck(compare_sort as fn(Vec<(String, u8)>) -> bool);
    }

    #[test]
    fn correct_radix_for_u8() {
        let num = 0x50u8;
        assert_eq!(Some(num), num.get_digit_at(0));
        assert_eq!(None, num.get_digit_at(1));
        assert_eq!(None, num.get_digit_at(5));
    }

    #[test]
    fn correct_radix_for_u16() {
        let num = 0x3050u16;
        assert_eq!(Some(0x30), num.get_digit_at(0));
        assert_eq!(Some(0x50), num.get_digit_at(1));
        assert_eq!(None, num.get_digit_at(2));
        assert_eq!(None, num.get_digit_at(5));
    }

    #[test]
    fn correct_radix_for_u32() {
        let num = 0x70103050u32;
        assert_eq!(Some(0x70), num.get_digit_at(0));
        assert_eq!(Some(0x10), num.get_digit_at(1));
        assert_eq!(Some(0x30), num.get_digit_at(2));
        assert_eq!(Some(0x50), num.get_digit_at(3));
        assert_eq!(None, num.get_digit_at(4));
        assert_eq!(None, num.get_digit_at(7));
    }

    #[test]
    fn correct_radix_for_u64() {
        let num = 0x2040608070103050u64;
        assert_eq!(Some(0x20), num.get_digit_at(0));
        assert_eq!(Some(0x40), num.get_digit_at(1));
        assert_eq!(Some(0x60), num.get_digit_at(2));
        assert_eq!(Some(0x80), num.get_digit_at(3));
        assert_eq!(Some(0x70), num.get_digit_at(4));
        assert_eq!(Some(0x10), num.get_digit_at(5));
        assert_eq!(Some(0x30), num.get_digit_at(6));
        assert_eq!(Some(0x50), num.get_digit_at(7));
        assert_eq!(None, num.get_digit_at(8));
        assert_eq!(None, num.get_digit_at(13));
    }
}
