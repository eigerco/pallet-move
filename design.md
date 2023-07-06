- [Introduction](#introduction)
- [Move language](#move-language)
- [Pontem Move fork](#pontem-move-fork)
  - [Introduction](#introduction-1)
  - [Why The Changes Were Needed](#why-the-changes-were-needed)
  - [What were the changes?](#what-were-the-changes)
    - [Making all MoveVM crates `no_std`.](#making-all-movevm-crates-no_std)
    - [Making all MoveVM crates build for the `wasm32-unknown-unknown` target.](#making-all-movevm-crates-build-for-the-wasm32-unknown-unknown-target)
    - [Changing address length from 20 to 32 bytes.](#changing-address-length-from-20-to-32-bytes)
- [Pontem MoveVM pallet](#pontem-movevm-pallet)



# Introduction




# Move language

# Pontem Move fork

## Introduction
The Pontem Network has adapted the Move language to work with the Substrate framework. To do this, some changes were made to Move's Virtual Machine (MoveVM). This document discusses why these changes were necessary and provides an overview of them.

## Why The Changes Were Needed
Move, originally developed for the Libra project, is known for its safety and efficiency. But, to make it work with Substrate, adjustments to MoveVM were needed. Substrate uses a WebAssembly (Wasm) environment, and this is key to understanding why changes to MoveVM were necessary.

There are a few reasons why Substrate uses WebAssembly:

1. Easy Upgrades: With WebAssembly, Substrate can update the blockchain's runtime logic without having to fork. This is because the Wasm code is stored on the blockchain itself. This makes upgrades smoother and keeps everything, including account balances and logic, in one place.

2. Works Everywhere: WebAssembly code can run on different platforms without change. This is great for ensuring Substrate-based blockchains can operate on various systems.

3. Efficient: WebAssembly is fast and lightweight. This is important for blockchain as you want things to run as efficiently as possible.

4. Language Flexibility: Using WebAssembly means that developers are not stuck with one programming language. They can use different languages like Rust or C++.

Because of these features, Substrate uses a WebAssembly environment. The Move language had to be adjusted to be compatible with this environment. The Pontem fork of MoveVM is essentially these adjustments.

## What were the changes?

### Making all MoveVM crates `no_std`.

The first group of commits that were added by Pontem after forking the original Move
repository deal with making all MoveVM crates [`no_std`][1]. `no_std` means that the crate
does not depend on the Rust standard library . This is important because the Rust standard library may not be available in the Wasm environment. This means that any crate that depends on the Rust standard library cannot be used in a Substrate pallet.

Apart from adding the crate-level `no_std` attribute, the following changes were made to the code:
* substituted the use of `std` with [`sp-std`][2] crate. `sp-std` is a Substrate crate that provides a subset of the Rust standard library that is compatible with the Substrate runtime.
* removed incompatible code

[1]: https://docs.rust-embedded.org/book/intro/no-std.html
[2]: https://crates.io/crates/sp-std

### Making all MoveVM crates build for the `wasm32-unknown-unknown` target.

Apart from setting the build target this also included CI configuration updates.

### Changing address length from 20 to 32 bytes.

Move address length had to be changed from 20 to 32 bytes to match the [Substrate address length][3]. This was done by changing the `LENGTH` constant in the `move-core-types` crate as well as updating some hard-coded addresses.

[3]: https://docs.substrate.io/reference/address-formats/

# Pontem MoveVM pallet

The Pontem MoveVM pallet has a form of a Cargo crate. It depends on the Pontem's MoveVM form located here: https://github.com/pontem-network/sp-move-vm and wraps it into a Substrate pallet using the `frame-support` crate.

