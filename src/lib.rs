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
        if vec.len() <= 50 {
            //|| depth > 2 {
            vec.sort_unstable();
            return;
        }
        let mut counts = [0usize; 257];
        for elem in vec.iter() {
            let radix_val = radix_for_str(elem, depth);
            counts[radix_val as usize] += 1;
        }
        let mut sum = 0usize;
        let mut offsets = [0usize; 257];
        for i in 0..256 {
            offsets[i as usize] = sum;
            sum += counts[i as usize];
        }
        let mut next_free = offsets.clone();

        let mut block = 0usize;
        let mut i = 0usize;
        while block < 255 {
            if i >= offsets[block + 1] as usize {
                block += 1;
            } else {
                let radix_val = radix_for_str(&vec[i as usize], depth) as usize;
                if radix_val == block {
                    i += 1;
                } else {
                    vec.swap(i as usize, next_free[radix_val] as usize);
                    next_free[radix_val] += 1;
                }
            }
        }
        for i in 0..254 {
            sort_req(
                &mut vec[offsets[i as usize] as usize..offsets[i as usize + 1] as usize],
                depth + 1,
            );
        }
        sort_req(&mut vec[offsets[254]..], depth + 1);
    }

    fn radix_for_str(s: &str, d: usize) -> u16 {
        if s.as_bytes().len() > d {
            (s.as_bytes()[d] as u16 + 1)
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
        fn compare_sort(mut strings: Vec<String>) -> bool {
            let mut copy = strings.clone();
            copy.sort_unstable();
            afsort::sort(&mut strings);
            strings == copy
        };
        QuickCheck::new().tests(10000).quickcheck(
            compare_sort as
                fn(Vec<String>) -> bool,
        );
    }

    fn strings_en() -> Vec<String> {
        let f = File::open("/usr/share/dict/american-english").unwrap();
        let b = BufReader::new(f);
        let mut strings = b.lines().map(|l| l.unwrap()).collect::<Vec<String>>();
        let mut rng = rand::thread_rng();
        rng.shuffle(&mut strings);
        strings.into_iter().take(100000).collect::<Vec<String>>()
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

}
