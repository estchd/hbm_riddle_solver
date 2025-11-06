#![feature(integer_atomics)]
#![feature(duration_millis_float)]

use std::ops::{BitAnd, BitXor};
use std::sync::atomic::{AtomicI64, AtomicU128, Ordering};
use std::time::{Instant};
use hexhex::hex;
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use sha2::Digest;

static HASHES: &'static [&'static str] = &[
    "41de5c372b0589bbdb80571e87efa95ea9e34b0d74c6005b8eab495b7afd9994",
    "31da6223a100ed348ceb3254ceab67c9cc102cb2a04ac24de0df3ef3479b1036"
];

// 15 Symbols per line

static ALLOWED_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyz ";

struct SignIterator {
    current: [[u8; 15]; 4],
    num_allowed_chars: u8,
    finished: bool,
}

impl SignIterator {
    fn new(num_allowed_chars: u8) -> Self {
        Self {
            current: [[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; 4],
            num_allowed_chars,
            finished: false,
        }
    }

    fn iterate(&mut self, line: usize, index: usize) -> bool {
        if line == 3 && index == 15 {
            return true;
        }

        self.current[line][index] = self.current[line][index] + 1;

        if self.current[line][index] > self.num_allowed_chars {
            return if index == 15 {
                self.current[line] = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

                self.iterate(line + 1, 0)
            } else {
                self.current[line][index] = 1;

                self.iterate(line, index + 1)
            }
        }

        false
    }
}

impl Iterator for SignIterator {
    type Item = [[u8; 15]; 4];

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let current = self.current;

        self.finished = self.iterate(0, 0);

        Some(current)
    }
}

fn generate(current: &[[u8; 15]; 4], allowed_chars: &[char]) -> [String; 4] {
    let mut strings: [String; 4] = [
        String::with_capacity(15),
        String::with_capacity(15),
        String::with_capacity(15),
        String::with_capacity(15)
    ];

    for i in 0..4 {
        for l in 0..15 {
            let value = current[i][l];

            if value == 0 {
                break;
            }

            strings[i].push(allowed_chars[(value - 1 )as usize]);
        }
    }

    strings
}


fn main() {
    let allowed_chars = ALLOWED_CHARS.chars().collect::<Vec<char>>();
    let possible_hashes: Vec<Vec<u8>> = HASHES.iter()
        .map(|hash| hexhex::decode(hash).unwrap())
        .collect();

    /*
    for i in 0..=0xFFu8 {
        let vec = vec![i];

        let string_3 = hex(&vec).to_string();

        let string_1 = format!("{:02x}", i);

        let int_byte = i as i32;
        let final_byte = (int_byte & 0xFF) + 256;
        let string_2 = format!("{:x}", final_byte)[1..].to_string();

        if string_1 != string_2 {
            panic!("Not equal: i: {}, string_1: {}, string_2: {}", i, string_1, string_2);
        }

        if string_1 != string_3 {
            panic!("Not equal: i: {}, string_1: {}, string_3: {}", i, string_1, string_3);
        }
    }
    */
    /*
    for i in 0..allowed_chars.len() {
        let mut rand_1 = Random::new();
        let mut rand_2 = Random::new();

        let string_1 = format!("{}", &allowed_chars[i]);

        let bytes_1 = string_1.as_bytes();

        let byte_1 = bytes_1[0];

        rand_1.set_seed(byte_1 as i64);
        rand_2.set_seed(byte_1 as i64);

        rand_1.next_int(0xFFFFFF);
        rand_2.next_int(0xFFFFFF);

        for l in 0..allowed_chars.len() {
            let string_2 = format!("{}", &allowed_chars[l]);

            let bytes_2 = string_2.as_bytes();

            let byte_2 = bytes_2[0];

            rand_1.set_seed(byte_2 as i64);

            let value = rand_2.next_int(0xFFFFFF) as i64 + byte_2 as i64;
            rand_2.set_seed(value);
            rand_2.set_seed(byte_2 as i64);

            let result_1 = rand_1.next_int(0xFFFFFF);
            let result_2 = rand_2.next_int(0xFFFFFF);

            if result_1 != result_2 {
                panic!("Methods are not equal: short: {}, long: {}", result_1, result_2);
            }
        }
    }
    */

    let sign_iterator = SignIterator::new(allowed_chars.len() as u8);

    let index: AtomicU128 = AtomicU128::new(0);

    let result = sign_iterator.par_bridge().into_par_iter().find_first(|sign| {
        let current_index = index.fetch_add(1, Ordering::Relaxed);
        let perf = current_index % 1000000 == 0;

        let start_instant = Instant::now();

        let sign = generate(sign, &allowed_chars);

        let sign_instant = Instant::now();

        if sign[0].len() == 0 || sign[1].len() == 0 || sign[2].len() == 0 || sign[3].len() == 0 {
            return false;
        }

        let result = smoosh(perf, &sign[0], &sign[1], &sign[2], &sign[3]);

        let result_instant = Instant::now();

        let mut found: bool = false;

        if result.len() != possible_hashes[0].len() {
            panic!("Invalid length: {}, expected: {}", result.len(), possible_hashes[0].len());
        }

        for possible_hash in &possible_hashes {
            if &result == possible_hash {
                found = true;
                break;
            }
        }

        let check_instant = Instant::now();

        if perf {
            let sign_time = sign_instant.duration_since(start_instant).as_nanos();
            let result_time = result_instant.duration_since(sign_instant).as_nanos();
            let check_time = check_instant.duration_since(result_instant).as_nanos();

            println!();
            println!("sign_time: {}ns", sign_time);
            println!("result_time: {}ns", result_time);
            println!("check_time: {}ns", check_time);
            println!()

            //println!("{}", current_index);
        }

        return found;
    });

    if let Some(result) = result {
        let text = generate(&result, &allowed_chars);

        println!("ANSWER FOUND, HOORAY!!!!");
        println!("ANSWER:");
        println!("Question 1: {}", text[0]);
        println!("Question 2: {}", text[1]);
        println!("Question 3: {}", text[2]);
        println!("Question 4: {}", text[3]);
        println!();

        let hash = smoosh(false, &text[0], &text[1], &text[2], &text[3]);
        println!("Correct Hash: {}", hex(hash));
    }
    else {
        eprintln!("NO ANSWER FOUND, CONGRATS FOR THE WASTED TIME");
    }
}


fn smoosh(perf: bool, line_1: &str, line_2: &str, line_3: &str, line_4: &str) -> Vec<u8> {
    if line_1.len() == 0 || line_2.len() == 0 || line_3.len() == 0 || line_4.len() == 0 {
        return vec![];
    }

    let start_instant = Instant::now();

    let b1 = line_1.as_bytes();
    let b2 = line_2.as_bytes();
    let b3 = line_3.as_bytes();
    let b4 = line_4.as_bytes();

    let bytes_instant = Instant::now();

    let mut random = Random::new();

    random.set_seed(b1[0] as i64);
    let rand_1 = random.next_int(0xFFFFFF);
    random.set_seed(b2[0] as i64);
    let rand_2 = random.next_int(0xFFFFFF);
    random.set_seed(b3[0] as i64);
    let rand_3 = random.next_int(0xFFFFFF);
    random.set_seed(b4[0] as i64);
    let rand_4 = random.next_int(0xFFFFFF);

    let rand_instant = Instant::now();

    let rand_1_string = format!("{}", rand_1);
    let rand_2_string = format!("{}", rand_2);
    let rand_3_string = format!("{}", rand_3);
    let rand_4_string = format!("{}", rand_4);

    let rand_string_instant = Instant::now();

    let mut s: String = String::with_capacity(
        line_1.len() +
            line_2.len() +
            line_3.len() +
            line_4.len() +
            rand_1_string.len() +
            rand_2_string.len() +
            rand_3_string.len() +
            rand_4_string.len()
    );

    s = s + line_1;
    s = s + &rand_1_string;
    s = s + line_2;
    s = s + &rand_2_string;
    s = s + line_3;
    s = s + &rand_3_string;
    s = s + line_4;
    s = s + &rand_4_string;

    let string_instant = Instant::now();

    let hash = get_hash(&s);

    let hash_instant = Instant::now();

    if perf {
        let bytes_time = bytes_instant.duration_since(start_instant).as_nanos();
        let rand_time = rand_instant.duration_since(bytes_instant).as_nanos();
        let rand_string_time = rand_string_instant.duration_since(rand_instant).as_nanos();
        let string_time = string_instant.duration_since(rand_string_instant).as_nanos();
        let hash_time = hash_instant.duration_since(string_instant).as_nanos();

        println!();
        println!("bytes_time: {}ns", bytes_time);
        println!("rand_time: {}ns", rand_time);
        println!("rand_string_time: {}ns", rand_string_time);
        println!("string_time: {}ns", string_time);
        println!("hash_time: {}ns", hash_time);
        println!()
    }

    hash
}

fn get_hash(input: &str) -> Vec<u8> {
    sha2::Sha256::digest(input.as_bytes()).to_vec()
}

static SEED_UNIQUIFIER: AtomicI64 = AtomicI64::new(8682522807148012);

fn seed_uniquifier() -> i64 {
    loop {
        let current: i64 = SEED_UNIQUIFIER.load(Ordering::Relaxed);

        let next : i64 = current.wrapping_mul(1181783497276652981);

        if SEED_UNIQUIFIER.compare_exchange(current, next, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            return next;
        }
    }
}

static MULTIPLIER: i64 = 0x5DEECE66D;
static ADDEND: i64 = 0xB;
static MASK: i64 = (1 << 48) - 1;

struct Random {
    seed: i64,
    have_next_next_gaussian: bool,
}

impl Random {
    pub fn new() -> Self {
        Self::from_seed(seed_uniquifier().bitxor(rand::rng().random_range(0..i64::MAX)))
    }

    pub fn from_seed(seed: i64) -> Self {
        Self {
            seed: Self::initial_scramble(seed),
            have_next_next_gaussian: false,
        }
    }

    fn initial_scramble(seed: i64) -> i64 {
        seed.bitxor(MULTIPLIER).bitand(MASK)
    }

    pub fn set_seed(&mut self, seed: i64) {
        self.seed = Self::initial_scramble(seed);
        self.have_next_next_gaussian = false;
    }

    pub fn next_int(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            panic!("Invalid Bound: {}", bound);
        }

        let mut r = self.next(31);
        let m = bound - 1;

        if bound.bitand(m) == 0 {
            let mul_bound = (bound as i64).wrapping_mul(r as i64);
            r = (mul_bound >> 31) as i32;
        }
        else {
            let mut u = r;

            loop {
                r = u % bound;

                let value = u.wrapping_sub(r).wrapping_add(m);

                if value >= 0 {
                    break;
                }

                u = self.next(31);
            }
        }

        r
    }

    fn next(&mut self, bits: i32) -> i32 {
        let oldseed: i64 = self.seed;
        let newseed: i64 = oldseed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND).bitand(MASK);

        self.seed = newseed;

        let shift_index: u32 = 48 - bits as u32;

        let shifted = newseed as u64 >> shift_index;

        shifted as i32
    }
}