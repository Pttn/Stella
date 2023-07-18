# Stella

**Stella** is a software written in Rust that finds *prime k-tuplets* (also called *prime constellations*). For now, it is just an inefficient prototype with very few features. It is currently provided as an experimental Rust Crate; we will assume that you are already familiar with this programming language.

One of it's goals is to become usable for Riecoin mining and possibly succeed [rieMiner](https://riecoin.dev/en/rieMiner) (Stella can be seen as a Rust port of rieMiner), and to break world records. Even if the Rust implementation were to remain less efficient, the improved code readability may help to try new optimizations that could then be implemented and ported back to rieMiner.

Another goal would be to help Riecoin and number crunching developers that are interested in understanding how the mining algorithm works. rieMiner is the merge and heavy refactor of previous Riecoin miners, and carries years of history, which can make the understanding of its code difficult, especially with the fact that documentation and explanations by previous developers are basically nonexistent. Stella is written from scratch and in a more modular way, which can greatly help the learning process for future developers.

## Stella Crate Usage

Since this is a prototype software, a lot of the usage instructions is subject to change in the future.

The Stella interface relies on the Rug's Integer structure, so add the following to your `Cargo.toml` in order to use the Crate:

```
rug = "^1.19.2"
stella = "0.0.3"
```

Optionally, you can use the following imports in your source files, we will assume that you did that below.

```
use rug::Integer;
use stella::Stella;
```

### Stella Instance

A `Stella` instance will handle the customizable search of Prime Constellations, create it with

```
let mut stella = Stella::new();
```

### Parameters

Now, the instance must be configured via a struct called `Params`, using the `set_params` method. Here are the fields of this structure:

* `workers: usize`: number of workers to use for the search. Set this to `0` or omit it to autodetect the number of threads in your machine;
* `constellation_pattern: Vec<isize>`: which sort of constellations to sieve for, as (cumulative) offsets separated by commas. Set this to an empty Vector or omit it to use the default pattern `0, 2, 6, 8, 12, 18, 20`;
* `prime_table_limit`: the prime table used for searching prime constellations will contain primes up to the given number. Set this to `0` or omit it to use the default limit of `16777216`;
* `primorial_number`: the Primorial Number for the sieve process. It should be such that the actual primorial is a few orders of magnitude smaller than the job targets. Set this to `0` or omit it to use the default value of `120`;
* `primorial_offset`: the offset from a primorial multiple to use for the sieve process. Set this `0` or omit it to choose automatically a hardcoded one associated to the pattern;
* `sieve_size`: the size of the primorial factors table for the sieve in bits. It will be rounded down to the previous multiple of the machine's word size if needed. Set this to `0`or omit it to use the default size of 2^25;

To be able to omit parameters, use `..Default::default()`. The chosen parameters should be suitable for the jobs that are going to be handled by the Stella instance. Here is an example of a configuration, compatible with the parameters above,

```
stella.set_params(stella::Params {
	workers: 8,
	constellation_pattern: vec![0, 2, 6, 8, 12, 18, 20, 26],
	prime_table_limit: 10000000,
	primorial_number: 100,
	sieve_size: 10000000,
	..Default::default() // Use this if you don't want to set some parameters (like primorial_offset here)
});
```

### Initialization

Once proper parameters have been set with `set_params`, the Stella instance must be initialized with

```
stella.init();
```

### Starting Workers

Start workers with

```
stella.start_workers();
```

This launches detached worker threads that will look for prime constellations once some valid jobs are added to the instance. Since the workers are detached threads, a main thread must also be run by the library user. In order to add jobs, view statistics, and handle results found by the Stella instance, read the sections below.

### Jobs

A job can be submitted to the Stella instance using the a struct called `Params` and the `add_job` method. Here are the fields of this structure:

* `id: usize`: an identifier for the job that must be unique;
* `clear_previous_jobs: bool`: whether to clear active jobs in the Stella instance;
* `pattern: Vec<isize>`: the target pattern for the outputs, which may differ from the one we are sieving for but must not be longer;
* `target_min: Integer`: the lower bound for the base prime number;
* `target_max: Integer`: the upper bound for the base prime number;
* `k_min: usize`: how many numbers in the target pattern must be prime in order for the tuple to be outputted;
* `pattern_min: Vec<bool>`: a vector that must be of the same size as the target pattern. True requires that the number at this position in the tuple is prime and false allows it to not be prime, in order for the tuple to be outputted.

All the fields must be set. The `add_job` method does some basic sanity checks and returns a couple of vectors of `String`s containing possible warnings or errors. If there were errors, the job is ignored by the Stella instance. Here is an usage example of the method and structure,

```
let (warnings, errors) = stella.add_job(stella::Job {
	id: 1,
	clear_previous_jobs: true,
	pattern: vec![0, 2, 6, 8, 12, 18, 20],
	target_min: Integer::from(1) << 1024,
	target_max: (Integer::from(1) << 1024) + (Integer::from(1) << 768),
	k_min: 5,
	pattern_min: vec![true ; 7]
});
```

### Stats

Once the Stella instance is initialized, you can access some relevant statistics with the `stats` method. It contains the following fields:

* `prime_table_size: usize`: number of prime numbers in the prime table;
* `prime_table_generation_time: f64`: how much time in s it took to generate this prime table;
* `modular_inverses_generation_time: f64`: how much time in s it took to generate the modular inverses table;
* `search_start_instant: Instant`: the instant when the workers were launched;
* `sieving_duration: f64`: the CPU time in s spent for sieving;
* `candidates_generated: usize`: how many candidates were generated during that time;
* `testing_duration: f64`: the CPU time in s spent for testing candidates;
* `tuple_counts: Vec<usize>`: how many tuples were found (the index is the tuple length associated to the count).

### Outputs

When a result fulfilling the job's conditions is found by the Stella instance, it is internally pushed to a queue. Using the `pop_output` method, you can retrieve an output from the queue and "consume" it. It is presented as an `Output` structure containing the following fields:

* `n: Integer`: the base number of the tuple;
* `pattern: Vec<isize>`: at which offsets of the target pattern the number is prime;
* `id: usize`: the job Id this output is associated to;
* `worker_id: usize`: the internal id of the worker that found the result.

### Example Program

An example program is provided in the GitHub repository and may be ran in the following way.

```bash
git clone https://github.com/Pttn/Stella.git
cd Stella
cargo build --release
```

You can inspect the `main.rs` source file to see a concrete example of the use of the Stella Crate.

## Developers and License

* [Pttn](https://github.com/Pttn), you can reach me on the Riecoin [Forum](https://forum.riecoin.dev/) or [Discord](https://discordapp.com/channels/525275069946003457) ([invite](https://discord.gg/2sJEayC)).

This work is released under the MIT license.

## Contributing

Feel free to make a pull request, and I will review it. By contributing to Stella, you accept to place your code under the MIT license.

Donations to the Riecoin Project are welcome:

* Riecoin: ric1pv3mxn0d5g59n6w6qkxdmavw767wgwqpg499xssqfkjfu5gjt0wjqkffwja
* Bitcoin: bc1pv3mxn0d5g59n6w6qkxdmavw767wgwqpg499xssqfkjfu5gjt0wjqej6g08

## Resources

* [Riecoin website](https://Riecoin.dev/)
* [Stella's Topic on the Riecoin Forum](https://forum.riecoin.dev/viewtopic.php?t=114)
* [Stella crate on Crates.io](https://crates.io/crates/Stella)
* [rieMiner's page](https://riecoin.dev/en/rieMiner)
* [Explanation of the miner's algorithm](https://riecoin.dev/en/Mining_Algorithm)
* [Stella, the Riecoin's PoW](https://riecoin.dev/en/Stella)
* [Stella, the Riecoin's Moe Personification](https://os-tans.moe/wiki/Stella)
