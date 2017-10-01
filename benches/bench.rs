#![feature(test)]

extern crate test;
extern crate rand;
extern crate regex;
extern crate afsort;

use test::Bencher;
use std::fs::File;
use std::io::{BufRead, BufReader};
use rand::Rng;
use regex::Regex;
use std::path::PathBuf;
use afsort::AFSortable;

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
