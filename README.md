# Stella

**Stella** is a software that finds *prime k-tuplets* (also called *prime constellations*). For now, it is just an inefficient prototype with very few features.

One of it's goals is to become usable for Riecoin mining and possibly succeed [rieMiner](https://riecoin.dev/en/rieMiner) (Stella can be seen as a Rust port of rieMiner), and to break world records. Even if the Rust implementation were to remain less efficient, the improved code readability may help to try new optimizations that could then be implemented and ported back to rieMiner.

Another goal would be to help Riecoin and number crunching developers that are interested in understanding how the mining algorithm works. rieMiner is the merge and heavy refactor of previous Riecoin miners, and carries years of history, which can make the understanding of its code difficult, especially with the fact that documentation and explanations by previous developers are basically nonexistent. Stella is written from scratch, which can greatly help the learning process for future developers.

## Build this program

Stella is written in **Rust** and uses **Cargo**, refer to the [installation chapter](https://doc.rust-lang.org/book/ch01-01-installation.html) of the Rust Book to learn how to install them on your system if needed. Depending on your OS, you may need to adjust some commands below.

Once done, clone the Stella's repository with Git in a terminal or download the Zip via the GitHub's web interface, change to the directory, and build the code.

```bash
git clone https://github.com/Pttn/Stella.git
cd Stella
cargo build --release
```

An executable is produced at `target/release/stella`, which can be ran either normally or with `cargo run --release`, but please read below first.

## Configure this program

Stella uses a text configuration file, by default a "Stella.conf" placed in the terminal's present working directory. It is also possible to use custom paths, examples with `cargo run`:

```bash
cargo run --release -- config/example.txt
cargo run --release -- "config 2.conf"
cargo run --release -- /home/user/Stella/rieMiner.conf"
```

or

```bash
cd target/release/
./stella ../../Stella.conf
./stella config/example.txt
./stella "config 2.conf"
./stella /home/user/Stella/rieMiner.conf
```

The syntax is very simple: each option is set by a line like

```
Option = Value
```

It is case sensitive. A line starting with `#` will be ignored, as well as invalid ones. Spaces or tabs just before or after `=` are also trimmed. If an option is missing, the default value(s) will be used. If there are duplicate lines for the same option, the last one will be used.

Alternatively, command line options can be used like

```bash
cargo run --release -- config.conf Option1=Value1 "Option2 = Value2" Option3=WeirdValue\!\!
./stella config.conf Option1=Value1 "Option2 = Value2" Option3=WeirdValue\!\!
```

A configuration file path must always be provided. If the file exists, its options will be parsed first, then the command line ones, so the latter will override the common ones from the file. Else, it is just ignored, so just put a dummy value if you want to configure only by command line. The syntax of a command line option is the same as a line of the configuration file. You are responsible for taking care of special characters if needed.

### Options

* `PrimeTableLimit`: the prime table used for searching prime constellations will contain primes up to the given number. Default: `16777216`;
* `ConstellationPattern`: which sort of constellations to look for, as offsets separated by commas. Note that they are not cumulative, so `0, 2, 4, 2, 4, 6, 2` corresponds to $n + (0, 2, 6, 8, 12, 18, 20)$. Default: `0, 2, 4, 2, 4, 6, 2`;
* `PrimorialNumber`: Primorial Number for the sieve process. Higher is better, but for practical reasons it should be such that the actual primorial is a bit smaller than the target. Default: `120`;
* `PrimorialOffset` or `PrimorialOffsets`: the offset from a primorial multiple to use for the sieve process. If not set, a default one will be chosen if one is hardcoded for the chosen constellation pattern;
* `SieveBits`: the size of the primorial factors table for the sieve is `2^SieveBits` bits. Default: `25`;
* `Difficulty`: sets the difficulty (which is the number of binary digits). It can take decimal values and the numbers will be around `2^Difficulty` (for now, only the floor of this value will be taken in consideration). Default: `1024`;
* `RefreshInterval`: refresh rate of the stats in seconds. Default: `1`.

## Developers and license

* [Pttn](https://github.com/Pttn), you can reach me on the Riecoin [Forum](https://forum.riecoin.dev/) or [Discord](https://discordapp.com/channels/525275069946003457) ([invite](https://discord.gg/2sJEayC)).

This work is released under the MIT license.

## Contributing

Feel free to make a pull request, and I will review it. By contributing to Stella, you accept to place your code under the MIT license.

Donations to the Riecoin Project are welcome:

* Riecoin: ric1qr3yxckxtl7lacvtuzhrdrtrlzvlydane2h37ja
* Bitcoin: bc1qr3yxckxtl7lacvtuzhrdrtrlzvlydaneqela0u

## Resources

* [Riecoin website](https://Riecoin.dev/)
* [Stella's Topic on the Riecoin Forum](https://forum.riecoin.dev/viewtopic.php?t=114)
* [rieMiner's page](https://riecoin.dev/en/rieMiner)
* [Explanation of the miner's algorithm](https://riecoin.dev/en/Mining_Algorithm)
* [Stella, the Riecoin's PoW](https://riecoin.dev/en/Stella)
* [Stella, the Riecoin's Moe Personification](https://os-tans.moe/wiki/Stella)
