- [Introduction](#introduction)
  - [Move language](#move-language)
  - [Substrate framework](#substrate-framework)
- [Pontem Move fork](#pontem-move-fork)
  - [Introduction](#introduction-1)
  - [Why The Changes Were Needed](#why-the-changes-were-needed)
  - [What were the changes?](#what-were-the-changes)
    - [Making all MoveVM crates `no_std`.](#making-all-movevm-crates-no_std)
    - [Making all MoveVM crates build for the `wasm32-unknown-unknown` target.](#making-all-movevm-crates-build-for-the-wasm32-unknown-unknown-target)
    - [Changing address length from 20 to 32 bytes.](#changing-address-length-from-20-to-32-bytes)
- [Pontem MoveVM pallet](#pontem-movevm-pallet)
  - [`GenesisConfig` and storage](#genesisconfig-and-storage)
  - [`SubstrateWeight`](#substrateweight)
  - [`rpc` crate](#rpc-crate)
  - [`runtime` crate](#runtime-crate)
- [The present state of Move VMs](#the-present-state-of-move-vms)
  - [Official repository](#official-repository)
  - [Available forks](#available-forks)
  - [Is forking needed?](#is-forking-needed)
  - [Forking challenges - VM and the toolchain](#forking-challenges---vm-and-the-toolchain)
- [The way forward](#the-way-forward)
  - [Proposed solution - the architecture](#proposed-solution---the-architecture)
  - [Deliverables](#deliverables)

# Introduction

## Move language

## Substrate framework

# Pontem Move fork

## Introduction
The Pontem Network has [adapted the Move language][0] to work with the Substrate framework. In order to do this, some adjustments were made to Move's Virtual Machine (MoveVM). This document provides an overview of all the necessary changes.

[0]: https://github.com/pontem-network/sp-move-vm

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
does not depend on the Rust standard library. This is important because the Rust standard library may not be available in the Wasm environment. This means that any crate that depends on the Rust standard library cannot be used in a Substrate pallet.

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

The [Pontem MoveVM pallet][4] has a form of a Cargo crate. It depends on the Pontem's MoveVM fork described in the previous section and wraps it into a Substrate pallet using the `frame-support` crate.

The crate exposes 3 main entry points: [`execute`][5], [`publish_module`][6], and [`publish_package`][7]. Each of them expects bytecode and `gas_limit` arguments.


[4]: https://github.com/pontem-network/pontem/tree/master/pallets/sp-mvm.
[5]: https://github.com/pontem-network/pontem/blob/master/pallets/sp-mvm/src/lib.rs#L188
[6]: https://github.com/pontem-network/pontem/blob/master/pallets/sp-mvm/src/lib.rs#L220
[7]: https://github.com/pontem-network/pontem/blob/master/pallets/sp-mvm/src/lib.rs#L252

## `GenesisConfig` and storage

Substrate allows you to configure the initial state of the blockchain by providing a [`GenesisConfig`][8]. The Pontem MoveVM pallet uses this to
set up its storage.


[8]: https://docs.substrate.io/build/genesis-configuration

## `SubstrateWeight`

Defines the weight of the pallet's functions. This is used by the `pallet::weight` macro to specify the weight of the extrinsics.

## `rpc` crate

This crate defines the runtime RPC made available by this pallet.

## `runtime` crate

Declares the `MVMApiRuntime` trait placed inside the [`sp_api::decl_runtime_apis!`][9] macro. The macro creates two declarations, one for use on the client side and one on the runtime side.

[9]: https://paritytech.github.io/substrate/master/sp_api/macro.decl_runtime_apis.html

# The present state of Move VMs

## Official repository

## Available forks

## Is forking needed?

## Forking challenges - VM and the toolchain

# The way forward

## Proposed solution - the architecture

## Deliverables


