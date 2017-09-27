#![feature(test)]
#[cfg(test)]
extern crate test;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate quickcheck;

pub mod afsort {

    pub fn sort(vec: &mut [String]) {
        sort_req(vec, 0);
    }

    fn sort_req(vec: &mut [String], depth: usize) {
        if vec.len() <= 32 {
            vec.sort_unstable();
            return;
        }
        let mut min = u16::max_value();
        let mut max = 0u16;
        for elem in vec.iter() {
            if elem.len() > depth {
                let val = elem.as_bytes()[depth] as u16;
                if val < min {
                    min = val;
                }
                if val > max {
                    max = val;
                }
            }
        }
        //No non-empty value found
        if min == u16::max_value() {
            return;
        }

        let num_items = (max - min + 2) as u16;
        let mut counts: Vec<usize> = vec![0usize; num_items as usize];

        for elem in vec.iter() {
            let radix_val = radix_for_str(elem, depth, min);
            counts[radix_val as usize] += 1;
        }

        let mut sum = 0usize;
        let mut offsets: Vec<usize> = vec![0usize; num_items as usize];
        for i in 0..num_items {
            offsets[i as usize] = sum;
            sum += counts[i as usize];
        }
        let mut next_free = offsets.clone();

        let mut block = 0u16;
        let mut i = 0usize;
        while block < num_items - 1 {
            if i >= offsets[block as usize + 1] as usize {
                block += 1;
            } else {
                let radix_val = radix_for_str(&vec[i as usize], depth, min);
                if radix_val == block {
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
                depth + 1,
            );
        }
        sort_req(&mut vec[offsets[num_items as usize - 1]..], depth + 1);
    }

    fn radix_for_str(s: &str, d: usize, base: u16) -> u16 {
        if s.as_bytes().len() > d {
            (s.as_bytes()[d] as u16 + 1 - base)
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

    #[test]
    fn sorts_same_as_unstable() {
        QuickCheck::new().tests(1000).quickcheck(
            compare_sort as
                fn(Vec<String>) -> bool,
        );
    }

    #[test]
    fn foobar() {
        assert!(compare_sort(
            vec![String::from("\u{40001}"), String::from("\u{80000}")],
        ))
    }

    fn compare_sort(mut strings: Vec<String>) -> bool {
        let mut copy = strings.clone();
        copy.sort_unstable();
        afsort::sort(&mut strings);
        strings == copy
    }

    #[bench]
    fn std_sort(b: &mut Bencher) {
        let strings = strings_en();
        b.iter(|| strings.clone().sort_unstable())
    }

    #[bench]
    fn af_sort(b: &mut Bencher) {
        let strings = strings_en();
        b.iter(|| afsort::sort(&mut strings.clone()))
    }

    fn strings_en() -> Vec<String> {
        let f = File::open("/usr/share/dict/american-english").unwrap();
        let b = BufReader::new(f);
        let mut strings = b.lines().map(|l| l.unwrap()).collect::<Vec<String>>();
        let mut rng = rand::thread_rng();
        rng.shuffle(&mut strings);
        strings.into_iter().take(10000).collect::<Vec<String>>()
    }


}
