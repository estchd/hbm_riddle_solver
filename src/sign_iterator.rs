
// 15 Symbols per line

use std::cmp::min;
use nonempty::NonEmpty;

static DEFAULT_BRUTE_FORCE_STATE: [u8; 15] = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

#[derive(Clone, Debug)]
enum IteratorLineConfig {
	BruteForce,
	Dictionary {
		possible_entries: Vec<[u8; 15]>
	},
	ConstantLine {
		constant: [u8; 15]
	}
}

impl IteratorLineConfig {
	pub fn from_readable_config(allowed_chars: &NonEmpty<char>, line_options: &Option<&[&str]>, line_constant: &Option<&str>) -> Self {
		if let Some(constant) = line_constant {
			return Self::ConstantLine {
				constant: Self::string_to_char_indices(allowed_chars, constant),
			}
		}

		if let Some(options) = line_options {
			let mut indexed_options: Vec<[u8; 15]> =  Vec::with_capacity(options.len());

			for option in *options {
				indexed_options.push(Self::string_to_char_indices(allowed_chars, option));
			}

			return Self::Dictionary {
				possible_entries: indexed_options,
			}
		}

		Self::BruteForce
	}

	fn string_to_char_indices(allowed_chars: &NonEmpty<char>, string: &str) -> [u8; 15] {
		let string_chars = string.chars().collect::<Vec<char>>();

		let mut char_indices = [0u8; 15];

		for i in 0..min(15, string_chars.len()) {
			for l in 0..allowed_chars.len() {
				if string_chars[i] == allowed_chars[l] {
					char_indices[i] = l as u8 + 1;
					break;
				}
			}
		}

		char_indices
	}
}

#[derive(Clone, Debug)]
enum IteratorLine {
	BruteForce {
		state: [u8; 15],
		num_possible_chars: u8
	},
	Dictionary {
		current_index: usize,
		possible_entries: Vec<[u8; 15]>
	},
	ConstantLine {
		constant: [u8; 15]
	}
}

impl IteratorLine {
	pub fn brute_force(num_possible_chars: u8) -> Self {
		Self::BruteForce {
			state: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			num_possible_chars
		}
	}

	pub fn dictionary(possible_entries: Vec<[u8; 15]>) -> Self {
		Self::Dictionary {
			current_index: 0,
			possible_entries,
		}
	}

	pub fn constant_line(constant: [u8; 15]) -> Self {
		Self::ConstantLine {
			constant,
		}
	}

	fn from_config(config: IteratorLineConfig, num_possible_chars: u8) -> Self {
		match config {
			IteratorLineConfig::BruteForce => {
				Self::brute_force(num_possible_chars)
			}
			IteratorLineConfig::Dictionary { possible_entries } => {
				Self::dictionary(possible_entries)
			}
			IteratorLineConfig::ConstantLine { constant } => {
				Self::constant_line(constant)
			}
		}
	}

	fn to_line(&self) -> [u8; 15] {
		match self {
			Self::BruteForce { state , num_possible_chars: _num_possible_chars } => {
				*state
			},
			IteratorLine::Dictionary { current_index, possible_entries } => {
				possible_entries[*current_index]
			},
			IteratorLine::ConstantLine { constant } => {
				*constant
			},
		}
	}

	fn is_brute_force(&self) -> bool {
		match self {
			Self::BruteForce { .. } => true,
			_ => false,
		}
	}

	fn is_dictionary(&self) -> bool {
		match self {
			Self::Dictionary { .. } => true,
			_ => false,
		}
	}

	fn is_constant(&self) -> bool {
		match self {
			Self::ConstantLine { .. } => true,
			_ => false,
		}
	}

	fn iterate(&mut self) -> bool {
		match self {
			IteratorLine::BruteForce { state, num_possible_chars } => {
				let mut index = 0;

				loop {
					state[index] = state[index] + 1;

					if state[index] > *num_possible_chars {
						if index == 15 {
							*state = DEFAULT_BRUTE_FORCE_STATE;
							return true;
						}
						else {
							state[index] = 1;
							index += 1;
						}
					}
					else {
						return false;
					}
				}
			},
			IteratorLine::Dictionary { current_index, possible_entries } => {
				*current_index = *current_index + 1;

				if *current_index == possible_entries.len() {
					*current_index = 0;
					return true;
				}

				false
			}
			IteratorLine::ConstantLine { .. } => {
				true
			}
		}
	}
}

pub struct SignIterator {
	lines: [IteratorLine; 4],
	initial: bool,
	only_constant: bool,
	only_dictionary_or_constant: bool,
	has_dictionary: bool,
}

impl SignIterator {
	fn from_config(config: [IteratorLineConfig; 4], allowed_chars: NonEmpty<char>) -> Self {
		let lines: [IteratorLine; 4] = config
			.into_iter()
			.map(|config| IteratorLine::from_config(config, allowed_chars.len() as u8))
			.collect::<Vec<IteratorLine>>()
			.try_into()
			.unwrap();

		let only_constant = lines
			.iter()
			.map(|line| line.is_constant())
			.fold(true, |acc, x| acc && x);

		let only_dictionary_or_constant = lines
			.iter()
			.map(|line| line.is_dictionary() || line.is_constant())
			.fold(true, |acc, x| acc && x);

		let has_dictionary = lines
			.iter()
			.map(|line| line.is_dictionary())
			.fold(false, |acc, x| acc || x);

		Self {
			lines,
			initial: true,
			only_constant,
			only_dictionary_or_constant,
			has_dictionary,
		}
	}

	pub fn from_readable_config(allowed_chars: NonEmpty<char>, line_options: &[Option<&[&str]>; 4], line_constant: &[Option<&str>; 4]) -> Self {
		let config = [
			IteratorLineConfig::from_readable_config(&allowed_chars, &line_options[0], &line_constant[0]),
			IteratorLineConfig::from_readable_config(&allowed_chars, &line_options[1], &line_constant[1]),
			IteratorLineConfig::from_readable_config(&allowed_chars, &line_options[2], &line_constant[2]),
			IteratorLineConfig::from_readable_config(&allowed_chars, &line_options[3], &line_constant[3]),
		];

		Self::from_config(config, allowed_chars)
	}

	fn iterate(&mut self) -> bool {
		if self.only_constant {
			return true;
		}

		if self.has_dictionary {
			let mut last_dictionary_line_finished = false;

			for i in 0..4 {
				if self.lines[i].is_dictionary() {
					last_dictionary_line_finished = self.lines[i].iterate();

					if !last_dictionary_line_finished {
						break;
					}
				}
			}

			if !last_dictionary_line_finished {
				return false;
			}

			if self.only_dictionary_or_constant {
				return true;
			}
		}

		for i in 0..4 {
			if self.lines[i].is_brute_force() && !self.lines[i].iterate() {
				return false;
			}
		}

		true
	}

	fn current(&self) -> [[u8; 15]; 4] {
		[
			self.lines[0].to_line(),
			self.lines[1].to_line(),
			self.lines[2].to_line(),
			self.lines[3].to_line(),
		]
	}
}

impl Iterator for SignIterator {
	type Item = [[u8; 15]; 4];

	fn next(&mut self) -> Option<Self::Item> {
		if self.initial {
			self.initial = false;
			return Some(self.current());
		}

		if self.iterate() {
			return None;
		}

		Some(self.current())
	}
}