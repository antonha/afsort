#![feature(test)]

extern crate afsort;
extern crate rand;
extern crate regex;
extern crate test;

use afsort::AFSortable;
use rand::Rng;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use test::Bencher;

#[bench]
fn sort_en_strings_rand_10_000_std(b: &mut Bencher) {
    let strings = strings_en(&Regex::new(r".*").unwrap(), 10_000);
    b.iter(|| strings.clone().sort_unstable())
}

#[bench]
fn sort_en_strings_rand_10_000_af(b: &mut Bencher) {
    let strings = strings_en(&Regex::new(r".*").unwrap(), 10_000);
    b.iter(|| strings.clone().af_sort_unstable())
}

#[bench]
fn sort_en_strings_rand_100_000_std(b: &mut Bencher) {
    let strings = strings_en(&Regex::new(r".*").unwrap(), 100_000);
    b.iter(|| strings.clone().sort_unstable())
}

#[bench]
fn sort_en_strings_rand_100_000_af(b: &mut Bencher) {
    let strings = strings_en(&Regex::new(r".*").unwrap(), 100_000);
    b.iter(|| strings.clone().af_sort_unstable())
}

#[bench]
fn sort_en_strings_sorted_10_000_std(b: &mut Bencher) {
    let mut strings = strings_en(&Regex::new(r".*").unwrap(), 10_000);
    strings.sort_unstable();
    b.iter(|| strings.clone().sort_unstable())
}

#[bench]
fn sort_en_strings_sorted_10_000_af(b: &mut Bencher) {
    let mut strings = strings_en(&Regex::new(r".*").unwrap(), 10_000);
    strings.sort_unstable();
    b.iter(|| strings.clone().af_sort_unstable())
}

#[bench]
fn sort_en_strings_lower_10_000_std(b: &mut Bencher) {
    let strings = strings_en(&Regex::new(r"^[a-z]+$").unwrap(), 10000);
    b.iter(|| strings.clone().sort_unstable())
}

#[bench]
fn sort_en_strings_lower_10_000_af(b: &mut Bencher) {
    let strings = strings_en(&Regex::new(r"^[a-z]+$").unwrap(), 10000);
    b.iter(|| strings.clone().af_sort_unstable())
}

#[bench]
fn sort_u8_1_000_std(b: &mut Bencher) {
    let nums = rand_u8(1_000);
    b.iter(|| nums.clone().sort_unstable())
}

#[bench]
fn sort_u8_1_000_af(b: &mut Bencher) {
    let nums = rand_u8(1_000);
    b.iter(|| nums.clone().af_sort_unstable())
}

#[bench]
fn sort_u16_1_000_000_std(b: &mut Bencher) {
    let nums = rand_u16(1_000_000);
    b.iter(|| nums.clone().sort_unstable())
}

#[bench]
fn sort_u16_1_000_000_af(b: &mut Bencher) {
    let nums = rand_u16(1_000_000);
    b.iter(|| nums.clone().af_sort_unstable())
}

#[bench]
fn sort_u32_1_000_000_std(b: &mut Bencher) {
    let nums = rand_u32(1_000_000);
    b.iter(|| nums.clone().sort_unstable())
}

#[bench]
fn sort_u32_1_000_000_af(b: &mut Bencher) {
    let nums = rand_u32(1_000_000);
    b.iter(|| nums.clone().af_sort_unstable())
}

#[bench]
fn sort_u64_1_000_000_std(b: &mut Bencher) {
    let nums = rand_u64(1_000_000);
    b.iter(|| nums.clone().sort_unstable())
}

#[bench]
fn sort_u64_1_000_000_af(b: &mut Bencher) {
    let nums = rand_u64(1_000_000);
    b.iter(|| nums.clone().af_sort_unstable())
}

fn rand_u8(n: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(rng.next_u32() as u8)
    }
    v
}

fn rand_u16(n: usize) -> Vec<u16> {
    let mut rng = rand::thread_rng();
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(rng.next_u32() as u16)
    }
    v
}

fn rand_u32(n: usize) -> Vec<u32> {
    let mut rng = rand::thread_rng();
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(rng.next_u32())
    }
    v
}

fn rand_u64(n: usize) -> Vec<u64> {
    let mut rng = rand::thread_rng();
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(rng.next_u64())
    }
    v
}

fn strings_en(re: &Regex, n: usize) -> Vec<String> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let f = File::open(d.join("test_resources/american-english.txt")).unwrap();
    let b = BufReader::new(f);
    let mut strings = b
        .lines()
        .map(|l| l.unwrap())
        .filter(|l| re.is_match(l))
        .collect::<Vec<String>>();
    let mut rng = rand::thread_rng();
    rng.shuffle(&mut strings);
    strings.into_iter().take(n).collect::<Vec<String>>()
}
