// (c) 2023 Pttn (Stelo.xyz/Riecoin.dev)

use rug::Integer;
use std::collections::VecDeque;
use std::mem::size_of;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::available_parallelism;
use std::time::Instant;

pub const WORD_SIZE: usize = 8*size_of::<usize>();

pub const DEFAULT_PRIMORIAL_OFFSETS: &'static [(&'static [isize], u128)] = &[
	(&[0], 380284918609481),
	(&[0, 2], 380284918609481),
	(&[0, 2, 6], 380284918609481),
	(&[0, 4, 6], 1418575498573),
	(&[0, 2, 6, 8], 380284918609481),
	(&[0, 2, 6, 8, 12], 380284918609481),
	(&[0, 4, 6, 10, 12], 1418575498597),
	(&[0, 4, 6, 10, 12, 16], 1091257),
	(&[0, 2, 6, 8, 12, 18, 20], 380284918609481),
	(&[0, 2, 8, 12, 14, 18, 20], 1418575498589),
	(&[0, 2, 6, 8, 12, 18, 20, 26], 380284918609481),
	(&[0, 2, 6, 12, 14, 20, 24, 26], 1418575498577),
	(&[0, 6, 8, 14, 18, 20, 24, 26], 1418575498583),
	(&[0, 2, 6, 8, 12, 18, 20, 26, 30], 380284918609481),
	(&[0, 2, 6, 12, 14, 20, 24, 26, 30], 1418575498577),
	(&[0, 4, 6, 10, 16, 18, 24, 28, 30], 1418575498573),
	(&[0, 4, 10, 12, 18, 22, 24, 28, 30], 1418575498579),
	(&[0, 2, 6, 8, 12, 18, 20, 26, 30, 32], 380284918609481),
	(&[0, 2, 6, 12, 14, 20, 24, 26, 30, 32], 1418575498577),
	(&[0, 2, 6, 8, 12, 18, 20, 26, 30, 32, 36], 380284918609481),
	(&[0, 4, 6, 10, 16, 18, 24, 28, 30, 34, 36], 1418575498573),
	(&[0, 2, 6, 8, 12, 18, 20, 26, 30, 32, 36, 42], 380284918609481),
	(&[0, 6, 10, 12, 16, 22, 24, 30, 34, 36, 40, 42], 1418575498567)
];

// Struct for results of interest found by a Stella instance (actual prime k-tuplet, long enough tuple, or pool share).
#[derive(Clone)]
pub struct Output {
	pub n: Integer,
	pub k: usize,
	pub constellation_pattern: Vec<isize>,
	pub worker_id: usize
}

// Struct containing parameters for a Stella instance.
#[derive(Clone)]
pub struct Params {
	pub workers: usize,
	pub constellation_pattern: Vec<isize>,
	pub target: Integer,
	pub prime_table_limit: usize,
	pub primorial_number: usize,
	pub primorial_offset: u128,
	pub sieve_size: usize,
}

impl Params {
	pub fn new() -> Params {
		return Params {
			workers: 0,
			constellation_pattern: vec![],
			target: Integer::from(0),
			prime_table_limit: 0,
			primorial_number: 0,
			primorial_offset: 0,
			sieve_size: 0
		};
	}
}

// Struct containing relevant statistics of a Stella instance.
#[derive(Clone)]
pub struct Stats {
	pub prime_table_size: usize,
	pub prime_table_generation_time: f64,
	pub modular_inverses_generation_time: f64,
	pub search_start_instant: Instant,
	pub sieving_duration: f64,
	pub candidates_generated: usize,
	pub testing_duration: f64,
	pub candidates_tested: usize,
	pub tuple_counts: Vec<usize>
}

impl Stats {
	pub fn new() -> Stats {
		return Stats {
			prime_table_size: 0,
			prime_table_generation_time: 0f64,
			modular_inverses_generation_time: 0f64,
			search_start_instant: Instant::now(),
			sieving_duration: 0f64,
			candidates_generated: 0,
			testing_duration: 0f64,
			candidates_tested: 0,
			tuple_counts: vec![]
		};
	}
}

// Struct used by workers for sieving.
struct Sieve {
	first_candidate: Integer,
	factors_eliminated: Vec<usize>,
	factors_candidates: Vec<usize>,
}

// Main structure for the library user, handles a customizable search of Prime Constellations.
pub struct Stella {
	params: Params,
	
	primes: Arc<Vec<usize>>,
	modular_inverses: Arc<Vec<usize>>,
	primorial: Integer,
	
	stats: Arc<Mutex<Stats>>,
	output: Arc<Mutex<VecDeque<Output>>>,
}

impl Stella {
	pub fn new() -> Stella {
		return Stella {
			params: Params::new(),
			primes: Arc::new(vec![]),
			modular_inverses: Arc::new(vec![]),
			primorial: Integer::from(1),
			stats: Arc::new(Mutex::new(Stats::new())),
			output: Arc::new(Mutex::new(VecDeque::new()))
		};
	}
	
	pub fn params(&self) -> Params {
		return self.params.clone();
	}
	
	pub fn set_params(&mut self, params: Params) -> () {
		if params.workers == 0 {
			self.params.workers = available_parallelism().unwrap().get();
		}
		
		else {
			self.params.workers = params.workers;
		}
		if params.constellation_pattern.len() == 0 { // Pick a default pattern if none was chosen
			self.params.constellation_pattern = vec![0, 2, 6, 8, 12, 18, 20];
			self.params.primorial_offset = DEFAULT_PRIMORIAL_OFFSETS.iter().find(|&&x| x.0 == &self.params.constellation_pattern).unwrap().1;
		}
		else {
			self.params.constellation_pattern = params.constellation_pattern;
		}
		
		if params.target == 0 {
			self.params.target = Integer::from(1) << 1024;
		}
		else {
			self.params.target = params.target;
		}
		
		if params.prime_table_limit == 0 {
			self.params.prime_table_limit = 16777216;
		}
		else {
			self.params.prime_table_limit = params.prime_table_limit;
		}
		
		if params.primorial_number == 0 {
			self.params.primorial_number = 120;
		}
		else {
			self.params.primorial_number = params.primorial_number;
		}
		
		if params.primorial_offset == 0 { // Pick a default Primorial Offset if none was chosen, if possible
			match DEFAULT_PRIMORIAL_OFFSETS.iter().find(|&&x| x.0 == &self.params.constellation_pattern) {
				Some(default_primorial_offset) => {self.params.primorial_offset = default_primorial_offset.1;}
				None => {panic!("The chosen Constellation Pattern does not have a default Primorial Offset, which must be set manually with the primorial_offset field.");}
			}
		}
		else {
			self.params.primorial_offset = params.primorial_offset;
		}
		
		if params.sieve_size == 0 {
			self.params.sieve_size = 1 << 25;
		}
		else {
			self.params.sieve_size = params.sieve_size;
		}
	}
	
	pub fn primorial(&self) -> Integer {
		return self.primorial.clone();
	}
	
	pub fn init(&mut self) -> () {
		let mut start_instant = Instant::now();
		self.primes = Arc::new(generate_primes(self.params.prime_table_limit));
		self.stats.lock().unwrap().prime_table_generation_time = time_since(start_instant);
		self.stats.lock().unwrap().prime_table_size = self.primes.len();
		self.primorial = primorial(&self.primes, self.params.primorial_number);
		start_instant = Instant::now();
		self.modular_inverses = Arc::new(compute_modular_inverses(&self.primorial, &self.primes));
		self.stats.lock().unwrap().modular_inverses_generation_time = time_since(start_instant);
	}
	
	pub fn start_workers(&mut self) -> () {
		let workers = self.params.workers;
		for worker_id in 0..workers {
			let primorial = self.primorial.clone();
			let primorial_offset = self.params.primorial_offset.clone();
			let params = self.params.clone();
			let constellation_pattern = self.params.constellation_pattern.clone();
			let primes = self.primes.clone();
			let modular_inverses = self.modular_inverses.clone();
			let sieve_size = self.params.sieve_size.clone();
			let sieve_words = sieve_size/WORD_SIZE;
			let output = self.output.clone();
			self.stats.lock().unwrap().search_start_instant = Instant::now();
			self.stats.lock().unwrap().sieving_duration = 0f64;
			self.stats.lock().unwrap().candidates_generated = 0;
			self.stats.lock().unwrap().testing_duration = 0f64;
			self.stats.lock().unwrap().candidates_tested = 0;
			self.stats.lock().unwrap().tuple_counts = vec![0; constellation_pattern.len() + 1];
			let stats = self.stats.clone();
			let target = self.params.target.clone() + worker_id*sieve_size*primorial.clone();
			let _ = thread::Builder::new().name(format!("Worker {0}", worker_id)).spawn(move || {
				// The first candidate is the first multiple of the primorial after the target + the primorial offset
				// The candidates have the form first_candidate + f × primorial
				let mut sieve = Sieve {
					first_candidate: target.clone() + primorial.clone() - (target.clone() % primorial.clone()) + primorial_offset,
					factors_eliminated: vec![0; sieve_words],
					factors_candidates: vec![]
				};
				let mut timer_instant;
				loop {
					timer_instant = Instant::now();
					// Eliminate the factors f
					for i in params.primorial_number .. primes.len() {
						for o in &constellation_pattern {
							let mut fp = (((primes[i] - ((sieve.first_candidate.clone() + o) % primes[i]))*modular_inverses[i]) % primes[i]).to_usize().unwrap();
							while fp < sieve_size {
								sieve.factors_eliminated[fp/WORD_SIZE] |= 1 << (fp % WORD_SIZE); // Mark as eliminated by changing the bit from 0 to 1 (if not already eliminated)
								fp += primes[i];
							}
						}
					}
					// Extract the factors from the sieve
					for i in params.primorial_number .. sieve.factors_eliminated.len() {
						let mut sieve_word = !sieve.factors_eliminated[i];
						while sieve_word != 0 {
							let n_eliminated_until_next = sieve_word.trailing_zeros() as usize;
							let candidate_factor = WORD_SIZE*i + n_eliminated_until_next;
							sieve.factors_candidates.push(candidate_factor);
							sieve_word &= sieve_word - 1; // Change the candidate's bit from 1 to 0.
						}
					}
					stats.lock().unwrap().sieving_duration += time_since(timer_instant);
					stats.lock().unwrap().candidates_generated += sieve.factors_candidates.len();
					timer_instant = Instant::now();
					// Check whether the candidates first_candidate + f × primorial are indeed prime constellations
					for i in 0 .. sieve.factors_candidates.len() {
						stats.lock().unwrap().tuple_counts[0] += 1;
						let mut k = 0;
						let candidate = &sieve.first_candidate + &Integer::from(sieve.factors_candidates[i])*primorial.clone();
						for o in &constellation_pattern {
							if is_prime_fermat(&(candidate.clone() + o)) {
								k += 1;
								stats.lock().unwrap().tuple_counts[k] += 1;
								if k >= constellation_pattern.len() {
									output.lock().unwrap().push_back(Output{
										n: candidate.clone(),
										k: k,
										constellation_pattern: constellation_pattern.clone(),
										worker_id: worker_id
									})
								}
							}
							else {
								break;
							}
						}
					}
					stats.lock().unwrap().testing_duration += time_since(timer_instant);
					stats.lock().unwrap().candidates_tested += sieve.factors_candidates.len();
					// Sieving and primality testing of the candidates finished, start over with new target
					sieve.first_candidate += primorial.clone()*sieve_size*workers;
					sieve.factors_eliminated = vec![0; sieve_words];
					sieve.factors_candidates = vec![];
				}
			});
		}
	}
	
	pub fn pop_output(&mut self) -> Option<Output> {
		return self.output.lock().unwrap().pop_back();
	}
	
	pub fn stats(&self) -> Stats {
		return self.stats.lock().unwrap().clone();
	}
}

// Measures how many s elapsed since the given instant
pub fn time_since(instant: Instant) -> f64 {
	return (instant.elapsed().as_nanos() as f64)/1_000_000_000f64
}

// Get Human Readable duration from an F64 storing the seconds
pub fn formatted_duration(duration : f64) -> String {
	if duration < 0.001 {return format!("{:.0} µs", 1000000f64*duration);}
	else if duration < 1f64 {return format!("{:.0} ms", 1000f64*duration);}
	else if duration < 60f64 {return format!("{:.2} s", duration);}
	else if duration < 3600f64 {return format!("{:.2} min", duration/60f64);}
	else if duration < 86400f64 {return format!("{:.2} h", duration/3600f64);}
	else if duration < 31556952f64 {return format!("{:.2} d", duration/86400f64);}
	else {return format!("{:.3} y", duration/31556952f64);}
}

// Generate all the prime numbers from 2 to limit inclusive with optimized Sieve of Eratosthenes (for 64 bits machines)
fn generate_primes(limit: usize) -> Vec<usize> {
	if limit < 2 {return Vec::new()};
	let mut composite_table: Vec<u64> = vec![0; limit/128 + 1]; // Booleans indicating whether an odd number is composite: 0000100100101100...
	let mut f = 3;
	while f*f <= limit { // Eliminate f and its multiples m for odd f from 3 to square root of the limit
		if composite_table[f >> 7] & (1 << ((f >> 1) & 63)) != 0 { // Skip if f is composite (f and its multiples were already eliminated)
			f += 2;
			continue;
		}
		let mut m = (f*f) >> 1;
		while m <= (limit >> 1) { // Start eliminating at f^2 (multiples of f below were already eliminated)
			composite_table[m >> 6] |= 1 << (m & 63);
			m += f;
		}
		f += 2;
	}
	
	let mut prime_table: Vec<usize> = vec![2];
	let mut i = 1;
	while (i << 1) + 1 <= limit { // Fill the prime table using the composite table
		if (composite_table[i >> 6] & (1 << (i & 63))) == 0 {
			prime_table.push((i << 1) + 1); // Add prime number 2i + 1
		}
		i += 1;
	}
	return prime_table;
}

// Computes the primorial_numberth primorial, a Vec containing enough prime numbers must be provided
fn primorial(primes: &Vec<usize>, primorial_number: usize) -> Integer {
	let mut primorial = Integer::from(1);
	for i in 1 .. primorial_number {
		primorial *= primes[i - 1];
	}
	return primorial;
}

// Computes the modular inverses a^(-1) of the integer a with respect to moduli m: a × a^(-1) ≡ 1 (mod m)
// Sets 0 if the inverse does not exist
fn compute_modular_inverses(a: &Integer, moduli: &Vec<usize>) -> Vec<usize> {
	let mut inverses = vec![0; moduli.len()];
	for i in 0 .. moduli.len() {
		match a.invert_ref(&Integer::from(moduli[i])) {
			Some(inverse) => inverses[i] = Integer::from(inverse).to_usize().unwrap(),
			_ => {}
		};
	}
	return inverses;
}

// n is probably prime if a^(n - 1) ≡ 1 (mod n) for one 0 < a < p or more (a = 2 is used here)
// Used for quick primality testing, outputs should be checked with an appropriate test.
fn is_prime_fermat(n: &Integer) -> bool {
	return Integer::from(2).pow_mod(&(n - Integer::from(1)), &n).unwrap() == 1;
}
