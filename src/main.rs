// (c) 2023 Pttn (Stelo.xyz/Riecoin.dev)

use rug::Integer;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::mem::size_of;
use std::process::exit;
use std::time::Instant;

struct SearchParameters {
	prime_table_limit: usize,
	constellation_pattern: Vec<usize>,
	primorial_number: usize,
	primorial_offset: u128,
	sieve_bits: usize,
	difficulty: f64,
	refresh_interval: f64
}

struct Stats {
	search_start_instant: Instant,
	tuple_count: Vec<usize>
}

struct Sieve {
	first_candidate: Integer,
	factors_eliminated: Vec<usize>,
	factors_candidates: Vec<usize>,
}

fn parse_options() -> SearchParameters {
	let mut search_parameters = SearchParameters {
		prime_table_limit: 16777216,
		constellation_pattern: vec![],
		primorial_number: 120,
		primorial_offset: 0,
		sieve_bits: 25,
		difficulty: 1024f64,
		refresh_interval: 1f64
	};
	
	let mut conf_path = "Stella.conf";
	
	let args: Vec<String> = env::args().collect();
	if args.len() >= 2 {
		conf_path = &args[1];
	}
	
	let mut lines = match fs::read_to_string(conf_path) {
		Ok(file_content) => {
			println!("Opening configuration file '{}'...", conf_path);
			let mut lines: Vec<String> = vec![];
			for line in file_content.lines() {
				lines.push(line.to_string());
			}
			lines
		}
		Err(e) => {
			if args.len() <= 2 && conf_path != "NoConf" {
				println!("'{}' not found or unreadable (reason: {}) and no other arguments given.", conf_path, e);
				exit(0);
			}
			vec![]
		}
	};
	
	if args.len() >= 2 {
		println!("Parsing {} option(s) given by command line...", args.len() - 2);
		for i in 2 .. args.len() {
			lines.push(args[i].clone());
		}
	}
	
	'lineLoop: for line in lines {
		if line.len() == 0 {continue;}
		if line.starts_with("#") {continue;}
		let option = line.split("=").collect::<Vec<_>>();
		if option.len() != 2 {
			println!("Ignoring invalid line '{}'", line);
			continue;
		}
		let key = option[0].trim_end();
		let value = option[1].trim_start();
		match key {
			"PrimeTableLimit" => match value.parse::<usize>() {
				Ok(prime_table_limit) => search_parameters.prime_table_limit = prime_table_limit,
				Err(_) => {println!("Invalid Value '{}' for Key '{}'", &value, &key);}
			}
			"ConstellationPattern" => {
				let offsets = value.split(",");
				let mut constellation_pattern = Vec::new();
				for offset in offsets {
					let offset = offset.trim();
					match offset.parse::<usize>() {
						Ok(offset) => constellation_pattern.push(offset),
						Err(_) => {
							println!("Invalid Value '{}' for Key '{}'", &value, &key);
							continue 'lineLoop;
						}
					}
				}
				search_parameters.constellation_pattern = constellation_pattern;
			}
			"PrimorialNumber" => match value.parse::<usize>() {
				Ok(primorial_number) => search_parameters.primorial_number = primorial_number,
				Err(_) => {println!("Invalid Value '{}' for Key '{}'", &value, &key);}
			}
			"PrimorialOffset" => match value.parse::<u128>() {
				Ok(primorial_offset) => search_parameters.primorial_offset = primorial_offset,
				Err(_) => {println!("Invalid Value '{}' for Key '{}'", &value, &key);}
			}
			"SieveBits" => match value.parse::<usize>() {
				Ok(sieve_bits) => search_parameters.sieve_bits = sieve_bits,
				Err(_) => {println!("Invalid Value '{}' for Key '{}'", &value, &key);}
			}
			"Difficulty" => match value.parse::<f64>() {
				Ok(difficulty) => search_parameters.difficulty = difficulty,
				Err(_) => {println!("Invalid Value '{}' for Key '{}'", &value, &key);}
			}
			"RefreshInterval" => match value.parse::<f64>() {
				Ok(refresh_interval) => search_parameters.refresh_interval = refresh_interval,
				Err(_) => {println!("Invalid Value '{}' for Key '{}'", &value, &key);}
			}
			&_ => {println!("Ignoring option with unused key '{}'", key);}
		}
	}
	
	return search_parameters;
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
fn is_prime_fermat(n: &Integer) -> bool {
	return Integer::from(2).pow_mod(&(n - Integer::from(1)), &n).unwrap() == 1;
}

// Measures how many s elapsed since the given instant
fn time_since(instant: Instant) -> f64 {
	return (instant.elapsed().as_nanos() as f64)/1_000_000_000f64
}

// Get Human Readable duration from an F64 storing the seconds
fn formatted_duration(duration : f64) -> String {
	if duration < 0.001 {return format!("{:.0} µs", 1000000f64*duration);}
	else if duration < 1f64 {return format!("{:.0} ms", 1000f64*duration);}
	else if duration < 60f64 {return format!("{:.2} s", duration);}
	else if duration < 3600f64 {return format!("{:.2} min", duration/60f64);}
	else if duration < 86400f64 {return format!("{:.2} h", duration/3600f64);}
	else if duration < 31556952f64 {return format!("{:.2} d", duration/86400f64);}
	else {return format!("{:.3} y", duration/31556952f64);}
}

fn main() {
	let mut default_primorial_offsets: HashMap<Vec<usize>, u128> = HashMap::new();
	default_primorial_offsets.insert(vec![0], 380284918609481);
	default_primorial_offsets.insert(vec![0, 2], 380284918609481);
	default_primorial_offsets.insert(vec![0, 2, 4], 380284918609481);
	default_primorial_offsets.insert(vec![0, 4, 2], 1418575498573);
	default_primorial_offsets.insert(vec![0, 2, 4, 2], 380284918609481);
	default_primorial_offsets.insert(vec![0, 2, 4, 2, 4], 380284918609481);
	default_primorial_offsets.insert(vec![0, 4, 2, 4, 2], 1418575498597);
	default_primorial_offsets.insert(vec![0, 4, 2, 4, 2, 4], 1091257);
	default_primorial_offsets.insert(vec![0, 2, 4, 2, 4, 6, 2], 380284918609481);
	default_primorial_offsets.insert(vec![0, 2, 6, 4, 2, 4, 2], 1418575498589);
	default_primorial_offsets.insert(vec![0, 2, 4, 2, 4, 6, 2, 6], 380284918609481);
	default_primorial_offsets.insert(vec![0, 2, 4, 6, 2, 6, 4, 2], 1418575498577);
	default_primorial_offsets.insert(vec![0, 6, 2, 6, 4, 2, 4, 2], 1418575498583);
	default_primorial_offsets.insert(vec![0, 2, 4, 2, 4, 6, 2, 6, 4], 380284918609481);
	default_primorial_offsets.insert(vec![0, 2, 4, 6, 2, 6, 4, 2, 4], 1418575498577);
	default_primorial_offsets.insert(vec![0, 4, 2, 4, 6, 2, 6, 4, 2], 1418575498573);
	default_primorial_offsets.insert(vec![0, 4, 6, 2, 6, 4, 2, 4, 2], 1418575498579);
	default_primorial_offsets.insert(vec![0, 2, 4, 2, 4, 6, 2, 6, 4, 2], 380284918609481);
	default_primorial_offsets.insert(vec![0, 2, 4, 6, 2, 6, 4, 2, 4, 2], 1418575498577);
	default_primorial_offsets.insert(vec![0, 2, 4, 2, 4, 6, 2, 6, 4, 2, 4], 380284918609481);
	default_primorial_offsets.insert(vec![0, 4, 2, 4, 6, 2, 6, 4, 2, 4, 2], 1418575498573);
	default_primorial_offsets.insert(vec![0, 2, 4, 2, 4, 6, 2, 6, 4, 2, 4, 6], 380284918609481);
	default_primorial_offsets.insert(vec![0, 6, 4, 2, 4, 6, 2, 6, 4, 2, 4, 2], 1418575498567);
	
	println!("Stella ES, by Pttn");
	println!("Repository: https://github.com/Pttn/Stella");
	println!("----------------------------------------------------------------");
	println!("Crates: Rug");
	println!("----------------------------------------------------------------");
	let mut search_parameters = parse_options();
	if search_parameters.constellation_pattern.len() == 0 { // Pick a default pattern if none was chosen
		search_parameters.constellation_pattern = vec![0, 2, 4, 2, 4, 6, 2];
		search_parameters.primorial_offset = default_primorial_offsets[&search_parameters.constellation_pattern];
	}
	if search_parameters.primorial_offset == 0 { // Pick a default Primorial Offset if none was chosen, if possible
		if default_primorial_offsets.contains_key(&search_parameters.constellation_pattern) {
			search_parameters.primorial_offset = default_primorial_offsets[&search_parameters.constellation_pattern];
		}
		else {
			println!("The chosen Constellation Pattern does not have a default Primorial Offset, which must be set manually with the PrimorialOffset option.");
			return;
		}
	}
	println!("Prime Table Limit: {}", search_parameters.prime_table_limit);
	println!("Constellation Pattern: {:?}", search_parameters.constellation_pattern);
	let mut constellation_pattern_cumulative = vec![];
	let mut sum = 0;
	for o in &search_parameters.constellation_pattern {
		sum += o;
		constellation_pattern_cumulative.push(sum);
	}
	println!("           Cumulative: {:?}", constellation_pattern_cumulative);
	println!("Primorial Number: {}", search_parameters.primorial_number);
	println!("Primorial Offset: {}", search_parameters.primorial_offset);
	let sieve_size = 1 << search_parameters.sieve_bits;
	let word_size = 8*size_of::<usize>();
	let sieve_words = sieve_size/word_size;
	println!("Sieve Bits: {} (words: {})", search_parameters.sieve_bits, sieve_words);
	println!("Difficulty: {}", search_parameters.difficulty);
	println!("Stats refresh interval: {} s", search_parameters.refresh_interval);
	println!("----------------------------------------------------------------");
	println!("Generating prime table using sieve of Eratosthenes...");
	let mut start_instant = Instant::now();
	let primes = generate_primes(search_parameters.prime_table_limit);
	let prime_table_generation_time = time_since(start_instant);
	println!("Table of {} primes generated in {:.6} s.", primes.len(), prime_table_generation_time);
	
	let primorial = primorial(&primes, search_parameters.primorial_number);
	if primorial < 1e18 {println!("Primorial: {primorial}");}
	else {println!("Primorial: {primorial:.12e}");}
	
	println!("Precomputing modular inverses...");
	start_instant = Instant::now();
	let modular_inverses = compute_modular_inverses(&primorial, &primes);
	let modular_inverses_generation_time = time_since(start_instant);
	println!("Table of modular inverses generated in {:.6} s.", modular_inverses_generation_time);
	println!("----------------------------------------------------------------");
	let target = Integer::from(1) << (search_parameters.difficulty.floor() as usize);
	if target < 1e18 {println!("Target: {target}");}
	else {println!("Target: {target:.12e}");}
	
	// The first candidate is the first multiple of the primorial after the target + the primorial offset
	// The candidates have the form first_candidate + f × primorial
	let mut sieve = Sieve {
		first_candidate: target.clone() + primorial.clone() - (target % primorial.clone()) + search_parameters.primorial_offset,
		factors_eliminated: vec![0; sieve_words],
		factors_candidates: vec![]
	};
	let mut stats = Stats {
		search_start_instant: Instant::now(),
		tuple_count: vec![0; search_parameters.constellation_pattern.len() + 1]
	};
	
	let mut timer_instant = Instant::now();
	println!("[{:.1}] Started Search", time_since(stats.search_start_instant));
	loop {
		start_instant = Instant::now();
		println!("[{:.1}] Sieving...", time_since(stats.search_start_instant));
		// Eliminate the factors f
		for i in search_parameters.primorial_number .. primes.len() {
			for o in &constellation_pattern_cumulative {
				let mut fp = (((primes[i] - ((sieve.first_candidate.clone() + o) % primes[i]))*modular_inverses[i]) % primes[i]).to_usize().unwrap();
				while fp < sieve_size {
					sieve.factors_eliminated[fp/word_size] |= 1 << (fp % word_size); // Mark as eliminated by changing the bit from 0 to 1 (if not already eliminated)
					fp += primes[i];
				}
			}
		}
		// Extract the factors from the sieve
		for i in search_parameters.primorial_number .. sieve.factors_eliminated.len() {
			let mut sieve_word = !sieve.factors_eliminated[i];
			while sieve_word != 0 {
				let n_eliminated_until_next = sieve_word.trailing_zeros() as usize;
				let candidate_factor = word_size*i + n_eliminated_until_next;
				sieve.factors_candidates.push(candidate_factor);
				sieve_word &= sieve_word - 1; // Change the candidate's bit from 1 to 0.
			}
		}
		let candidate_generation_time = time_since(start_instant);
		println!("[{:.1}] {} candidates found in {:.3} s ({:.1} per s)", time_since(stats.search_start_instant), sieve.factors_candidates.len(), candidate_generation_time, {sieve.factors_candidates.len() as f64}/candidate_generation_time);
		println!("[{:.1}] Primality checking...", time_since(stats.search_start_instant));
		start_instant = Instant::now();
		// Check whether the candidates first_candidate + f × primorial are indeed prime constellations
		for i in 0 .. sieve.factors_candidates.len() {
			stats.tuple_count[0] += 1;
			let mut k = 0;
			let candidate = &sieve.first_candidate + &Integer::from(sieve.factors_candidates[i])*primorial.clone();
			for o in &constellation_pattern_cumulative {
				if is_prime_fermat(&(candidate.clone() + o)) {
					k += 1;
					stats.tuple_count[k] += 1;
					if k >= search_parameters.constellation_pattern.len() {
						println!("{}-tuple found: {} + {:?}", k, candidate, constellation_pattern_cumulative);
					}
				}
				else {
					break;
				}
			}
			// Print Stats
			if time_since(timer_instant) > search_parameters.refresh_interval {
				let duration = time_since(stats.search_start_instant);
				let cps = (stats.tuple_count[0] as f64)/time_since(stats.search_start_instant);
				if stats.tuple_count[1] > 0 {
					let r = (stats.tuple_count[0] as f64)/(stats.tuple_count[1] as f64);
					let estimated_average_find_time = r.powf(search_parameters.constellation_pattern.len() as f64)/cps;
					println!("[{:.1}] {:.1} c/s, r: {:.2}, t: {:?} | {}", duration, cps, r, stats.tuple_count, formatted_duration(estimated_average_find_time));
				}
				else {
					println!("[{:.1}] {:.1} c/s, r: -.--, t: {:?}", duration, (stats.tuple_count[0] as f64)/time_since(stats.search_start_instant), stats.tuple_count);
				}
				timer_instant = Instant::now();
			}
		}
		let primality_testing_time = time_since(start_instant);
		println!("[{:.1}] Candidates tested in {:.3} s ({:.1} per s)", time_since(stats.search_start_instant), primality_testing_time, (sieve.factors_candidates.len() as f64)/primality_testing_time);
		println!("[{:.1}] Total {:.3} s, {:.1} effective candidates per s", time_since(stats.search_start_instant), candidate_generation_time + primality_testing_time, (sieve.factors_candidates.len() as f64)/(candidate_generation_time + primality_testing_time));
		// Sieving and primality testing of the candidates finished, start over with new target (which comes just after the largest candidate for the just finished sieve)
		sieve.first_candidate += primorial.clone()*sieve_size;
		sieve.factors_eliminated = vec![0; sieve_words];
		sieve.factors_candidates = vec![];
	}
}
