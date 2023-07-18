// (c) 2023 Pttn (Stelo.xyz/Riecoin.dev)

use rug::Integer;
use std::thread;
use std::time::{Duration, Instant};
use stella::Stella;
use stella::{formatted_duration, time_since};

fn main() {
	println!("Stella Demo App, by Pttn");
	println!("Repository: https://github.com/Pttn/Stella");
	println!("Crate: https://crates.io/crates/stella");
	println!("----------------------------------------------------------------");
	// Create a Stella instance
	let mut stella = Stella::new();
	// Configure the Stella instance with parameters suitable for the jobs it will take.
	stella.set_params(stella::Params {
		workers: 8,
		constellation_pattern: vec![0, 2, 6, 8, 12, 18, 20, 26], // Sieve Candidates for this Pattern
		prime_table_limit: 10000000,
		primorial_number: 100,
		// primorial_offset: 0,
		sieve_size: 10000000,
		..Default::default() // Use this if you don't want to set some parameters (like primorial_offset here)
	});
	// Check the Parameters
	let params = stella.params();
	println!("Workers: {}", params.workers);
	println!("Constellation Pattern: {:?}", params.constellation_pattern);
	println!("Prime Table Limit: {}", params.prime_table_limit);
	println!("Primorial Number: {}", params.primorial_number);
	println!("Primorial Offset: {}", params.primorial_offset);
	println!("Sieve Size: {} (words: {})", params.sieve_size, params.sieve_size/stella::WORD_SIZE);
	println!("----------------------------------------------------------------");
	// Initialize the Stella instance (Generate Prime Table and Modular Inverses,...)
	println!("Initializing the Stella instance...");
	stella.init();
	let stats = stella.stats();
	println!("Table of {} primes generated in {:.6} s.", stats.prime_table_size, stats.prime_table_generation_time);
	println!("Table of modular inverses generated in {:.6} s.", stats.modular_inverses_generation_time);
	let primorial = stella.primorial();
	if primorial < 1e18 {println!("Primorial: {0}", primorial);}
	else {println!("Primorial: {0:.12e}", primorial);}
	println!("----------------------------------------------------------------");
	println!("[{:.1}] Started Search", time_since(stella.stats().search_start_instant));
	// Start Worker Threads
	stella.start_workers();
	// Add a Job and check for possible issues with it
	let (warnings, errors) = stella.add_job(stella::Job {
		id: 1,
		clear_previous_jobs: true,
		pattern: (&params.constellation_pattern[0 .. params.constellation_pattern.len() - 1]).to_vec(), // Check Candidates for this pattern
		target_min: Integer::from(1) << 1024,
		target_max: (Integer::from(1) << 1024) + (Integer::from(1) << 768),
		k_min: params.constellation_pattern.len() - 2,
		pattern_min: vec![true ; params.constellation_pattern.len() - 1] // All true: stop checking a Candidate as soon as one of the number is not prime
		// pattern_min: vec![true, true, false, false, false, false, false] // Use something like this if doing Riecoin Pooled Mining
	});
	if !warnings.is_empty() {
		println!("Warnings(s): {:?}", warnings);
	}
	if !errors.is_empty() {
		println!("Error(s): {:?}", errors);
	}
	// Manage Worker Threads, Show Stats, Handle Outputs...
	let refresh_interval = 5f64;
	let mut timer = Instant::now();
	loop {
		let stats = stella.stats();
		let duration = time_since(stats.search_start_instant);
		// Poll possible Outputs
		loop {
			match stella.pop_output() {
				Some(output) => {
					println!("[{:.1}] {}-tuple found by thread {}: {} + {:?}", duration, output.pattern.len(), output.worker_id, output.n, output.pattern);
				},
				None => break
			}
		}
		// Get and Print Stats
		if time_since(timer) > refresh_interval {
			let cps = (stats.tuple_counts[0] as f64)/time_since(stats.search_start_instant);
			if stats.tuple_counts[1] > 0 {
				let r = (stats.tuple_counts[0] as f64)/(stats.tuple_counts[1] as f64);
				let estimated_average_find_time = r.powf(params.constellation_pattern.len() as f64)/cps;
				println!("[{:.1}] {:.1} c/s, r: {:.2}, t: {:?} | {}", duration, cps, r, stats.tuple_counts, formatted_duration(estimated_average_find_time));
				println!("[{:.1}] Sieving speed: {} candidates generated during {:.2} s of sieving: {:.1} candidates/s (CPU Time)", duration, stats.candidates_generated, stats.sieving_duration, (stats.candidates_generated as f64)/stats.sieving_duration);
				println!("[{:.1}] Testing speed: {} candidates checked during {:.2} s of primality testing: {:.1} candidates/s (CPU Time)", duration, stats.candidates_tested, stats.testing_duration, (stats.candidates_tested as f64)/stats.testing_duration);
			}
			else {
				println!("[{:.1}] {:.1} c/s, r: -.--, t: {:?}", duration, (stats.tuple_counts[0] as f64)/time_since(stats.search_start_instant), stats.tuple_counts);
			}
			timer = Instant::now();
		}
		thread::sleep(Duration::from_millis(100));
	}
}
