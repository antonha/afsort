#![feature(test)]
#[cfg(test)]
extern crate test;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
extern crate regex;

pub mod afsort {

    pub fn sort<'a, T>(vec: &'a mut [T])
    where
        T: AsRef<[u8]>,
    {
        sort_by(vec, (|s| s.as_ref()));
    }

    pub fn sort_by<T, F>(vec: &mut [T], to_slice: F)
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
            afsort::sort(&mut strings);
            strings == copy
        }
        QuickCheck::new().tests(1000).quickcheck(
            compare_sort as
                fn(Vec<String>) -> bool,
        );
    }

    #[test]
    fn sorts_tuples_same_as_unstable() {
        fn compare_sort(mut tuples: Vec<(String, u8)>) -> bool {
            let mut copy = tuples.clone();
            copy.sort_unstable_by(|t1, t2| t1.0.cmp(&t2.0));
            afsort::sort_by(&mut tuples, |t| t.0.as_bytes());
            tuples.into_iter().map(|t| t.0).collect::<Vec<String>>() ==
                copy.into_iter().map(|t| t.0).collect::<Vec<String>>()
        }
        QuickCheck::new().tests(1000).quickcheck(
            compare_sort as
                fn(Vec<(String, u8)>) -> bool,
        );
    }



    #[bench]
    fn std_sort(b: &mut Bencher) {
        let strings = strings_en(&Regex::new(r".*").unwrap());
        b.iter(|| strings.clone().sort_unstable())
    }

    #[bench]
    fn af_sort(b: &mut Bencher) {
        let strings = strings_en(&Regex::new(r".*").unwrap());
        b.iter(|| afsort::sort(&mut strings.clone()))
    }

    #[bench]
    fn std_sort_sorted(b: &mut Bencher) {
        let mut strings = strings_en(&Regex::new(r".*").unwrap());
        strings.sort_unstable();
        b.iter(|| strings.clone().sort_unstable())
    }

    #[bench]
    fn af_sort_sorted(b: &mut Bencher) {
        let mut strings = strings_en(&Regex::new(r".*").unwrap());
        strings.sort_unstable();
        b.iter(|| afsort::sort(&mut strings.clone()))
    }

    #[bench]
    fn std_sort_only_lower(b: &mut Bencher) {
        let strings = strings_en(&Regex::new(r"^[a-z]+$").unwrap());
        b.iter(|| strings.clone().sort_unstable())
    }

    #[bench]
    fn af_sort_only_lower(b: &mut Bencher) {
        let strings = strings_en(&Regex::new(r"^[a-z]+$").unwrap());
        b.iter(|| afsort::sort(&mut strings.clone()))
    }

    fn strings_en(re: &Regex) -> Vec<String> {
        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let f = File::open(d.join("test_resources/american-english.txt")).unwrap();
        let b = BufReader::new(f);
        let mut strings = b.lines()
            .map(|l| l.unwrap())
            .filter(|l| re.is_match(l))
            .collect::<Vec<String>>();
        let mut rng = rand::thread_rng();
        rng.shuffle(&mut strings);
        strings.into_iter().take(10000).collect::<Vec<String>>()
    }


}
