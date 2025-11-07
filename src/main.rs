#![feature(integer_atomics)]
#![feature(duration_millis_float)]

mod sign_iterator;

#[cfg(feature = "filter_dictionary")]
use std::fs::File;
#[cfg(feature = "filter_dictionary")]
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
#[cfg(feature = "filter_dictionary")]
use std::path::Path;

use std::ops::{BitAnd, BitXor};
use std::sync::atomic::{AtomicI64, Ordering};
use hexhex::hex;
use nonempty::NonEmpty;
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sha2::Digest;
use crate::sign_iterator::{SignIterator};

static HASHES: &'static [&'static str] = &[
    "41de5c372b0589bbdb80571e87efa95ea9e34b0d74c6005b8eab495b7afd9994",
    "31da6223a100ed348ceb3254ceab67c9cc102cb2a04ac24de0df3ef3479b1036"
];

static ALLOWED_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyz -#'123456789_,.";

/*
static LINE_CONSTANTS: [Option<&'static str>; 4] = [
    None,
    None,
    None,
    None
];
 */

//static ALL_LINE_OPTIONS: &'static [&'static str] = &["the void", "void", "popbub", "hoofington", "sharon", "flim flam industries", "enolagay", "enola gay", "lildip", "lil dip", "duchess gambit","duchessgambit", "hoboy03new", "celestium industries", "free electron laser", "sound", "air", "scream", "voice", "screen", "fourier", "ears", "ear", "music", "sound", "current", "voltage", "separation of isotopes by laser exitation", "chaos", "starcontrol", "radar", "numbernine", "dyx", "minelittlepony", "pisp", "tile.obj_tester.name", "exposure chamber", "fel", "free electron laser", "maxwell", "atmosphere", "Doctor Schrabauer", "DrNostalgia", "ffi-brand cigarette", "matter", "the world", "world", "mask man", "maskman", "balls-o-tron", "radon", "isotopes", "orbitals", "electrons", "atoms", "amber", "ambers", "flame", "balefire", "smoke ring", "cigarette smoke", "cigar smoke", "tobacco smoke", "smoke", "fire", "who", "java", "the author", "author", "the bobcat", "bob", "electrons", "photons", "decoder", "time", "seed", "hash", "you", "entropy", "operator", "nature", "scientist", "the observer", "observer", "euphemia li britannia", "digamma", "digamma crystal", "digamma laser crystal", "electricity", "current", "silex", "murky anvil", "capitalism", "smog", "xrays", "x-ray", "xray", "x-rays", "electromagnetic", "electromagnetism", "infrared", "microwaves", "fallout", "decoy", "belief", "skybox", "faith", "ignorance", "illusion","radio", "noise", "smog", "glare", "skyglow", "light pollution", "sunlight", "sun-rays", "wind", "uv-ray", "uv ray", "uv-rays", "uv rays", "gravity", "radiation", "clouds", "half-life scientists", "scientists", "you", "yourself", "mountains", "an echo", "echo", ];

/*
static LINE_OPTIONS: [Option<&[&'static str]>; 4] = [
    //None,
    Some(&["half-life scientists", "scientists", "you", "yourself", "mountains", "an echo", "echo"]),
    //None,
    Some(&["capitalism", "smog", "xrays", "x-ray", "xray", "x-rays", "electromagnetic", "electromagnetism", "infrared", "microwaves", "fallout", "decoy", "belief", "skybox", "faith", "ignorance", "illusion","radio", "noise", "smog", "glare", "skyglow", "light pollution", "sunlight", "sun-rays", "wind", "uv-ray", "uv ray", "uv-rays", "uv rays", "gravity", "radiation", "clouds"]),
    //None,
    Some(&["who", "java", "the author", "author", "the bobcat", "bob", "electrons", "photons", "decoder", "time", "seed", "hash", "you", "entropy", "operator", "nature", "scientist", "the observer", "observer", "euphemia li britannia", "digamma crystal", "digamma laser crystal", "electricity", "current", "silex", "murky anvil"]),
    //None,
    Some(&["mask man", "maskman", "balls-o-tron", "radon", "isotopes", "orbitals", "electrons", "atoms", "amber", "ambers", "flame", "balefire", "smoke ring", "cigarette smoke", "cigar smoke", "tobacco smoke", "smoke", "fire"])
];
 */

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn generate(current: &[[u8; 15]; 4], allowed_chars: &NonEmpty<u8>) -> [Vec<u8>; 4] {
    let mut strings: [Vec<u8>; 4] = [
        Vec::with_capacity(15),
        Vec::with_capacity(15),
        Vec::with_capacity(15),
        Vec::with_capacity(15)
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

#[cfg(feature = "filter_dictionary")]
fn filter_dictionary<A: AsRef<Path>, B: AsRef<Path>>(input_path: A, output_path: B) {
    let allowed_chars: NonEmpty<char> = NonEmpty::from_vec(ALLOWED_CHARS
        .chars()
        .collect::<Vec<char>>()).unwrap();

    let mut input = File::open(input_path.as_ref()).unwrap();

    let end = input.seek(SeekFrom::End(0)).unwrap();
    input.seek(SeekFrom::Start(0)).unwrap();

    let mut data: Vec<u8> = Vec::with_capacity(end as usize);

    let actual_size = input.read_to_end(&mut data).unwrap();

    data.truncate(actual_size);

    println!("loaded");

    let input = BufReader::new(data.as_slice());

    let lines = input.lines().count();

    println!("lines: {}", lines);

    let input = BufReader::new(data.as_slice());

    let mut output = BufWriter::new(File::create(output_path).unwrap());

    let mut index = 0;

    for line in input.lines() {
        let line = line.unwrap();

        index += 1;

        if index % 10000000 == 0 {
            println!("{}", index);
        }

        let line = line.chars().take(15);

        for char in line.clone() {
            if !allowed_chars.contains(&char) {
                continue;
            }
        }

        output.write_fmt(format_args!("{}\n", line.collect::<String>())).unwrap();
    }
}

#[cfg_attr(feature = "hotpath", hotpath::main(percentiles = [100], timeout = 1000))]
fn main() {
    #[cfg(feature = "filter_dictionary")]
    {
        filter_dictionary("length_dictionary.txt", "filtered_dictionary.txt");

        return;
    }

    let allowed_chars: NonEmpty<char> = NonEmpty::from_vec(ALLOWED_CHARS
        .chars()
        .collect::<Vec<char>>()).unwrap();

    let allowed_chars_bytes: NonEmpty<u8> =  NonEmpty::from_vec(allowed_chars
        .iter()
        .map(|char| {
            format!("{}", char).into_bytes()[0]
        })
        .collect::<Vec<u8>>()).unwrap();

    let possible_hashes: Vec<Vec<u8>> = HASHES.iter()
        .map(|hash| hexhex::decode(hash).unwrap())
        .collect();

    let char_random_hashes = allowed_chars_bytes
        .iter()
        .map(|char| {
            let mut random = Random::new();
            random.set_seed(*char as i64);

            let random_value = random.next_int(0xFFFFFF);



            format!("{}", random_value).into_bytes()
        })
        .collect::<Vec<Vec<u8>>>();

    let sign_iterator = SignIterator::from_readable_config(allowed_chars, &[None; 4], &[None; 4]);
    //let sign_iterator = SignIterator::from_readable_config(allowed_chars, &options, &LINE_CONSTANTS);

    let iterators = sign_iterator.split::<16>();

    let result = iterators.into_par_iter().find_map_first(|mut iterator| {
            iterator.find(|sign_indices| {
            check_solution(sign_indices, &allowed_chars_bytes, &possible_hashes, &char_random_hashes)
        })
    });

    if let Some(result) = result {
        let text = generate(&result, &allowed_chars_bytes);

        let text_1 = String::from_utf8(text[0].clone()).unwrap();
        let text_2 = String::from_utf8(text[1].clone()).unwrap();
        let text_3 = String::from_utf8(text[2].clone()).unwrap();
        let text_4 = String::from_utf8(text[3].clone()).unwrap();

        println!("ANSWER FOUND, HOORAY!!!!");
        println!("ANSWER:");
        println!("Question 1: {}", text_1);
        println!("Question 2: {}", text_2);
        println!("Question 3: {}", text_3);
        println!("Question 4: {}", text_4);
        println!();

        let line_1 = Line {
            string_bytes: &text[0],
            char_random_string_bytes: &char_random_hashes[(result[0][0] - 1) as usize],
        };

        let line_2 = Line {
            string_bytes: &text[1],
            char_random_string_bytes: &char_random_hashes[(result[1][0] - 1) as usize],
        };

        let line_3 = Line {
            string_bytes: &text[2],
            char_random_string_bytes: &char_random_hashes[(result[2][0] - 1) as usize],
        };

        let line_4 = Line {
            string_bytes: &text[3],
            char_random_string_bytes: &char_random_hashes[(result[3][0] - 1) as usize],
        };

        let hash = smoosh([line_1, line_2, line_3, line_4]);
        println!("Correct Hash: {}", hex(hash));
    }
    else {
        eprintln!("NO ANSWER FOUND, CONGRATS FOR THE WASTED TIME");
    }
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn check_solution(sign_indices: &[[u8; 15]; 4], allowed_chars: &NonEmpty<u8>, possible_hashes: &Vec<Vec<u8>>, char_random_hashes: &Vec<Vec<u8>>) -> bool {
    let sign = generate(sign_indices, &allowed_chars);

    if sign[0].len() == 0 || sign[1].len() == 0 || sign[2].len() == 0 || sign[3].len() == 0 {
        return false;
    }

    let line_1 = Line {
        string_bytes: &sign[0],
        char_random_string_bytes: &char_random_hashes[(sign_indices[0][0] - 1) as usize],
    };

    let line_2 = Line {
        string_bytes: &sign[1],
        char_random_string_bytes: &char_random_hashes[(sign_indices[1][0] - 1) as usize],
    };

    let line_3 = Line {
        string_bytes: &sign[2],
        char_random_string_bytes: &char_random_hashes[(sign_indices[2][0] - 1) as usize],
    };

    let line_4 = Line {
        string_bytes: &sign[3],
        char_random_string_bytes: &char_random_hashes[(sign_indices[3][0] - 1) as usize],
    };

    let result = smoosh([line_1, line_2, line_3, line_4]);

    let mut found: bool = false;

    if result.len() != possible_hashes[0].len() {
        panic!("Invalid length: {}, expected: {}", result.len(), possible_hashes[0].len());
    }

    for possible_hash in possible_hashes {
        if &result == possible_hash {
            found = true;
            break;
        }
    }

    found
}

struct Line<'a, 'b> {
    string_bytes: &'a [u8],
    char_random_string_bytes: &'b [u8],
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn smoosh(lines: [Line; 4]) -> Vec<u8> {
    if  lines[0].string_bytes.len() == 0 ||
        lines[1].string_bytes.len() == 0 ||
        lines[2].string_bytes.len() == 0 ||
        lines[3].string_bytes.len() == 0
    {
        return vec![];
    }

    let mut s: Vec<u8>;

    #[cfg(not(feature = "hotpath"))]
    {
        let len =
            lines[0].string_bytes.len() +
            lines[1].string_bytes.len() +
            lines[2].string_bytes.len() +
            lines[3].string_bytes.len() +
            lines[0].char_random_string_bytes.len() +
            lines[1].char_random_string_bytes.len() +
            lines[2].char_random_string_bytes.len() +
            lines[3].char_random_string_bytes.len();

        s = vec![0u8; len];

        s.extend_from_slice(lines[0].string_bytes);
        s.extend_from_slice(lines[0].char_random_string_bytes);
        s.extend_from_slice(lines[1].string_bytes);
        s.extend_from_slice(lines[1].char_random_string_bytes);
        s.extend_from_slice(lines[2].string_bytes);
        s.extend_from_slice(lines[2].char_random_string_bytes);
        s.extend_from_slice(lines[3].string_bytes);
        s.extend_from_slice(lines[3].char_random_string_bytes);
    }

    #[cfg(feature = "hotpath")]
    hotpath::measure_block!("riddle_solver::smoosh::input_string", {
       let len =
            lines[0].string_bytes.len() +
            lines[1].string_bytes.len() +
            lines[2].string_bytes.len() +
            lines[3].string_bytes.len() +
            lines[0].char_random_string_bytes.len() +
            lines[1].char_random_string_bytes.len() +
            lines[2].char_random_string_bytes.len() +
            lines[3].char_random_string_bytes.len();

        s = Vec::with_capacity(len);

        s.extend_from_slice(lines[0].string_bytes);
        s.extend_from_slice(lines[0].char_random_string_bytes);
        s.extend_from_slice(lines[1].string_bytes);
        s.extend_from_slice(lines[1].char_random_string_bytes);
        s.extend_from_slice(lines[2].string_bytes);
        s.extend_from_slice(lines[2].char_random_string_bytes);
        s.extend_from_slice(lines[3].string_bytes);
        s.extend_from_slice(lines[3].char_random_string_bytes);
    });

    let hash = get_hash(&s);

    #[cfg(feature = "debug_hash")]
    {
        println!();
        println!("{}", String::from_utf8(s.clone()).unwrap());
        println!("{}", hex(&hash));
        println!();
    }

    hash
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn get_hash(input: &[u8]) -> Vec<u8> {
    sha2::Sha256::digest(input).to_vec()
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