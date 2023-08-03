- [Introduction](#introduction)
  - [Move language](#move-language)
  - [Substrate framework](#substrate-framework)
- [The present state of Move VMs](#the-present-state-of-move-vms)
  - [Available forks](#available-forks)
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
- [The way forward](#the-way-forward)
  - [Is forking needed?](#is-forking-needed)
  - [Forking challenges - VM and the toolchain](#forking-challenges---vm-and-the-toolchain)
  - [Proposed solution - the architecture](#proposed-solution---the-architecture)
  - [MoveVM module and changes](#movevm-module-and-changes)
  - [Substrate MoveVM pallet](#substrate-movevm-pallet)
  - [Deliverables](#deliverables)

# Introduction
This article describes the ability to incorporate Move Virtual Machine into Substrate-based chains as a runtime module. It also describes the current state of the Move VM (with available forks, including Pontem work) and the challenges that need to be overcome to make it work with Substrate.

## Move language
Move is a programming language originally developed at Facebook to power the Diem blockchain. Its main aim was to give the ability to write smart contracts that can be run on the specialized virtual machine (Move VM) inside the blockchain. 

Move is a statically-typed language with a syntax that is similar to Rust. It introduces a slightly different resource handling concept where resources can never be copied or implicitly discarded - they can be moved (as the language name states) between program storage locations.

There were 4 main design goals for the Move language:
* First-Class Resources - one of the key features of Move is the ability to create custom resource types that can be handled safely, which is enforced by the static type system and move semantics.
* Flexibility - language introduces transaction scripts, which allow executing procedures with Move code which allows customizable transactions.
* Safety - Move is designed to be safe by default. It rejects all programs that violate Move's key properties, such as resource safety, memory safety, and type safety. It's achieved by checking the Move bytecode on-chain for safety by the bytecode verifier and, if passed, executed directly by the bytecode interpreter.
* Verifiability - there is an approach to perform as much lightweight on-chain verification as possible but support more complex verification off-chain, which can be performed by static verification tools. There have been several decisions made that make Move static verification friendly, like no dynamic dispatch, limited mutability and modularity support.

Programs (smart contracts) written in Move language are deployed as a bytecode and executed by the Move VM, which is a stack-based virtual machine. It has been designed to be simple, efficient and platform-agnostic, which means it's possible to integrate with custom blockchains or even run it separately and interact using a command line interface.

Move has been used as the smart-contract language for many blockchains like Sui, Starcoin, Aptos or Diem.

More information about the Move language can be found:
* [Move language whitepaper](https://diem-developers-components.netlify.app/papers/diem-move-a-language-with-programmable-resources/2020-05-26.pdf)
* [Move official repository](https://github.com/move-language/move)
* [The Move book](https://move-language.github.io/move/)
* [Move examples and papers](https://github.com/MystenLabs/awesome-move)

## Substrate framework
Substrate is a framework (SDK) which provides building tools for custom blockchains. Its main goal is to provide an environment that allows building blockchains with your own logic and features without the need to write everything from scratch. It's written in Rust and provides extensive documentation and usage samples.

Substrate is a modular framework which means it's possible to use only parts of it that are needed for the project. It provides a set of ready-to-use modules that can be used to build a blockchain. Substrate node consists of two general parts:
- a core client with node services (peer discovery, managing transaction requests, responding to RPC calls),
- a runtime which contains all the business logic of the blockchain.

The runtime is responsible for determining the state of the blockchain and processing all requested changes, including validation. The runtime module is designed to compile to WebAssembly and allows it to be extended by modules called pallets developed for the FRAME (Framework for Runtime Aggregation of Modularized Entities) subsystem. Since Move Virtual Machine updates a blockchain state, it should be part of the Substrate's runtime execution. Therefore, it should be provided as a pallet that can be loaded as a module.

More information about the Substrate framework can be found:
* [Official webpage](https://substrate.io/)

# The present state of Move VMs

The official repository for the Move language in the [move-language](https://github.com/move-language/move/tree/main/language) repo ([move-vm](https://github.com/move-language/move/tree/main/language/move-vm)). The open-source community maintains it, and also other Move forks keep it up to date.

## Available forks

The most important move-lang forks are:

 - [Aptos](https://github.com/aptos-labs/aptos-core/tree/main/third_party/move) ([move-vm](https://github.com/aptos-labs/aptos-core/tree/main/third_party/move/move-vm))
 - [Starcoin](https://github.com/starcoinorg/starcoin/tree/master/vm) ([move-vm](https://github.com/starcoinorg/starcoin/tree/master/vm/vm-runtime))
 - [0L](https://github.com/0LNetworkCommunity/libra/tree/v6/diem-move) ([move-vm](https://github.com/0LNetworkCommunity/libra/tree/v6/diem-move/diem-vm))
 - [Sui](https://github.com/MystenLabs/sui/tree/main/external-crates/move) ([move-vm](https://github.com/MystenLabs/sui/tree/main/external-crates/move/move-vm))

Maybe the most important of the above is the Sui's fork, a special dialect of the Move language called Sui Move. The key differences with the Diem-style Move language are:

- Sui uses its own object-centric global storage
- Addresses represent Object IDs
- Sui objects have globally unique IDs
- Sui has module initializers (init)
- Sui entry points take object references as input

The main constraint of Sui Move is that it is tightly coupled with the Sui blockchain - it's less flexible and not so platform agnostic, so it is currently not a candidate fork to be ported to a Substrate pallet.

# Pontem Move fork

## Introduction
The Pontem Network has [adapted the Move language][0] to work with the Substrate framework. In order to do this, some adjustments were made to Move's Virtual Machine (MoveVM). This document provides an overview of all the necessary changes.

[0]: https://github.com/pontem-network/sp-move-vm

## Why The Changes Were Needed
Move, originally developed for the Libra project, is known for its safety and efficiency. But, to make it work with Substrate, adjustments to MoveVM were needed. Substrate uses a WebAssembly (Wasm) environment, and this is key to understanding why changes to MoveVM were necessary.

There are a few reasons why Substrate uses WebAssembly:

1. Easy Upgrades: With WebAssembly, Substrate can update the blockchain's runtime logic without having to fork. This is because the Wasm code is stored on the blockchain itself. This makes upgrades smoother and keeps everything, including account balances and logic, in one place.

2. Works Everywhere: WebAssembly code can run on different platforms without change. This is great for ensuring Substrate-based blockchains can operate on various systems.

3. Efficient: WebAssembly is fast and lightweight. This is important for blockchain, as you want things to run as efficiently as possible.

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

Apart from setting the build target, this also included CI configuration updates.

### Changing address length from 20 to 32 bytes.

Move address length had to be changed from 20 to 32 bytes to match the [Substrate address length][3]. This was done by changing the `LENGTH` constant in the `move-core-types` crate as well as updating some hard-coded addresses. In our fork, there won't be a need to make any code changes because the move-lang codebase introduced support for the `address32` feature, which is today used by Aptos and Sui blockchains.

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

# The way forward

TODO: make some intro here

## Is forking needed?

TODO notes:
_Yes!_
- _Mention the `address32` move-lang feature we are going to use_
- _we need to apply no-std support throughout the codebase_

## Forking challenges - VM and the toolchain
As the forking is still needed there are some challenges that need to be addressed for the VM itself as well as for the toolchain.

One of the biggest challenges is to keep the possibility to use already existing contracts written in Move language. Two main goals can be achieved:
- level 1: source compatibility - keep the possibility to use existing contracts written in Move language after recompiling by modified toolchain; 
- level 2: bytecode compatibility - keep the possibility to use existing contracts in bytecode form.

Level 1 compatibility will be achieved, but at this point, we cannot guarantee level 2 compatibility. Level 2 depends on many external factors, like concrete blockchain features (address length, gas metering, etc.).

TODO: write here about the toolchain changes, ABI etc. - anything that can cause incompatibility with the existing contracts.

Another important challenge is keeping the repository in sync state with the upstream and separating custom work from the main line, which can allow us to perform upgrades easily. We aim to provide the least possible changes to the original codebase to provide a better upgrade experience and the ability to follow the mainline with all the language and feature updates. Of course, it will be not possible to keep everything separated from the main repository and therefore, some changes will be applied to the virtual machine itself.

Our proposal involves creating a fork of the Move VM `main` repository branch and do the changes on different branches. It will allow us to keep a clean and easy way to import changes from the upstream and apply them to our work. Many changes that are not affecting the modified codebase will be able to merge instantly (eg. by rebasing our branches to the new main head). Only mainline changes touching our modified code need to be applied manually.

The key point is to perform such minor updates periodically when it would be easier to maintain the changing codebase. Waiting too long between syncs can cause a lot of problems and make it impossible to merge changes without a lot of manual work, which can lead to a situation where it will be easier to fork the repository again and start from scratch, especially if it is done by the separate team than the one who created the fork. 

## Proposed solution - the architecture
The architecture of software represents a crucial design element that goes beyond the lines of code, empowering developers to visualize, plan, and construct robust and scalable solutions. However, our solution architecture is constrained by two external factors:
- The Substrate framework architecture.
- The Move language virtual machine architecture.
Both of these factors are out of our control, and we have to work with them as they are, fulfilling their requirements and fitting our solution into them. Moreover, some inspiration can be taken from the Pontem work on the MoveVM pallet as we've identified some of their design decisions to be good and that can be reused in our solution. 

After deep analysis, we've made decomposed the solution into smaller, manageable, and cohesive modules. Each module represents a distinct unit of functionality that can be developed, tested, and maintained mostly independently. The modules are:
1) The MoveVM module - described in [MoveVM module and changes](#movevm-module-and-changes).
2) The MoveVM Substrate pallet - described in [Substrate MoveVM pallet](#substrate-movevm-pallet):
   - The MoveVM pallet main library.
   - The MoveVM pallet runtime API.
   - The MoveVM pallet RPC.
3) The MoveVM testing Substrate node.
4) The MoveVM testing tools.

TODO: write further design description

## MoveVM module and changes

TODO: describe MoveVM architecture and all the changes that are needed to be done in the VM codebase.

## Substrate MoveVM pallet

TODO: describe here the pallet itself, its architecture and how it works.

## Testing
Software testing is an essential process in the software development life cycle that helps identify bugs, defects, and errors in a software application. Tests are performed from the very start of the project and each design decision takes into account the testability of the code. Testing is performed on different levels, from unit tests to integration tests and end-to-end tests. More about it can be found in the [Testing Guide](testing_guide.md) document which is one of the project deliverables.

## Repository structure
In order to keep things separated and easy to maintain, we propose to create a separate repository for each forked codebase and a separate repository for the VM pallet itself.

The package and repository structure will look like this:
- `pallet-move` - the [MoveVM pallet repository](https://github.com/eigerco/pallet-move) - contains the pallet codebase, tests, and documentation. Work is done in the `main` branch.
- `pallet-move-rpc` - the RPC MoveVM pallet repository - placed under `src/rpc` directory in the `pallet-move` repository. Contains the RPC codebase, tests, and documentation. Work is done in the `main` branch.
- `pallet-move-runtime-api` - the runtime API MoveVM pallet repository - placed under `src/rpc/runtime-api` directory in the `pallet-move` repository. Work is done in the `main` branch.
- `substrate-move` - Move language fork [repository](https://github.com/eigerco/substrate-move) - contains the Move language codebase, tests, and documentation.
- `substrate-node-template-move-vm-test` - testing [node repository](https://github.com/eigerco/substrate-node-template-move-vm-test) - contains the node codebase, tests, and documentation. Work is done in the `polkadot-1.0.0-pallet-move` branch.

Testing code should be separated from the actual codebase. That's a little different approach when compared to the previous Pontem work, where the Move pallet and machine were placed inside the main Pontem repository and built together with the node. Pontem implemented a real node and a usable blockchain, and therefore, it was justified to keep everything in one place. In our case, we are not going to build a full blockchain, but only a MoveVM pallet. We plan to use only a modified template node to prove our solution is working correctly. We believe it will be easier for further pallet integrators to have it separated from the node codebase as they can fork only the pallet repo and integrate it with their solutions.

## Deliverables

TODO: describe here what we are going to deliver.