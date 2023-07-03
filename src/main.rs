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
	// Configure the Stella instance
	/*stella.set_params(stella::Params { // Use Default Parameters
		workers: 0,
		constellation_pattern: vec![],
		target: Integer::from(0),
		prime_table_limit: 0,
		primorial_number: 0,
		primorial_offset: 0,
		sieve_size: 0
	});*/
	stella.set_params(stella::Params { // Use concrete Parameters
		workers: 8,
		constellation_pattern: vec![0, 2, 6, 8, 12, 18, 20, 26],
		prime_table_limit: 10000000,
		primorial_number: 20,
		primorial_offset: 0,
		target: Integer::from(1) << 128,
		sieve_size: 10000000
	});
	// Check the Parameters
	let params = stella.params();
	println!("Workers: {}", params.workers);
	println!("Constellation Pattern: {:?}", params.constellation_pattern);
	if params.target < 1e18 {println!("Target: {}", params.target);}
	else {println!("Target: {:.12e}", params.target);}
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
	if params.target < 1e18 {println!("Target: {}", params.target);}
	else {println!("Target: {:.12e}", params.target);}
	let primorial = stella.primorial();
	if primorial < 1e18 {println!("Primorial: {0}", primorial);}
	else {println!("Primorial: {0:.12e}", primorial);}
	println!("----------------------------------------------------------------");
	println!("[{:.1}] Started Search", time_since(stella.stats().search_start_instant));
	// Start Worker Threads
	stella.start_workers();
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
					println!("[{:.1}] {}-tuple found by thread {}: {} + {:?}", duration, output.k, output.worker_id, output.n, output.constellation_pattern);
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
