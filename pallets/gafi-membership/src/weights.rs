
//! Autogenerated weights for `gafi_membership`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-08-03, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/gafi-node
// benchmark
// pallet
// --chain
// dev
// --wasm-execution
// compiled
// --pallet
// gafi_membership
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --output
// ./pallets/src/gafi-membership/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn registration(s: u32,) -> Weight;
	fn remove_member(s: u32,) -> Weight;
}


/// Weight functions for `gafi_membership`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: GafiMembership MemberCount (r:1 w:1)
	// Storage: GafiMembership Members (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn registration(s: u32, ) -> Weight {
		(14_722_000_u64)
			// Standard Error: 1_000
			.saturating_add((1_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	// Storage: GafiMembership MemberCount (r:1 w:1)
	// Storage: GafiMembership Members (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn remove_member(_s: u32, ) -> Weight {
		(15_436_000_u64)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
}

impl WeightInfo for () {

	fn registration(s: u32, ) -> Weight {
		(14_722_000_u64)
			// Standard Error: 1_000
			.saturating_add((1_000 as Weight).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}

	fn remove_member(_s: u32, ) -> Weight {
		(15_436_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
}
