// This file is part of Gafi Network.

// Copyright (C) 2021-2022 Grindy Technologies.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
use crate::weights::WeightInfo;
use frame_support::{
	dispatch::{DispatchResult, Vec},
	pallet_prelude::*,
	traits::{
		tokens::{ExistenceRequirement, WithdrawReasons},
		Currency, ReservableCurrency,
	},
	transactional,
};
use frame_system::pallet_prelude::*;
use gafi_primitives::{
	constant::ID,
	pool::MasterPool,
	system_services::{SystemDefaultServices, SystemPool, SystemService},
	ticket::{Ticket, TicketType},
};
use gu_convertor::{u128_to_balance, u128_try_to_balance};
pub use pallet::*;
use pallet_timestamp::{self as timestamp};
use sp_runtime::traits::StaticLookup;

pub mod migration;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use gafi_primitives::players::PlayersTime;

	use super::*;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config + timestamp::Config + pallet_balances::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Synchronization remove_player by call remove_player on Master Pool
		type MasterPool: MasterPool<Self::AccountId>;

		/// Max number of player can join the Upfront Pool
		#[pallet::constant]
		type MaxPlayerStorage: Get<u32>;

		type UpfrontServices: SystemDefaultServices;

		type Players: PlayersTime<Self::AccountId>;
	}

	/// on_finalize following by steps:
	/// 1. Check if current timestamp is the correct time to charge service fee
	///	2. Charge player in the IngamePlayers - Kick player when they can't pay
	///	3. Move all players from NewPlayer to IngamePlayers
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_block_number: BlockNumberFor<T>) {
			let _now: u128 = Self::get_timestamp();
			if _now - T::MasterPool::get_marktime() >= T::MasterPool::get_timeservice() {
				let _ = Self::charge_ingame();
				let _ = Self::move_newplayer_to_ingame();
				Self::deposit_event(<Event<T>>::ChargePoolService);
			}
		}
	}

	//** Storage **//

	/// Holding the number of Max Player can join the Upfront Pool
	#[pallet::storage]
	#[pallet::getter(fn max_player)]
	pub type MaxPlayer<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Count player on the pool to make sure not exceed the MaxPlayer
	#[pallet::storage]
	#[pallet::getter(fn player_count)]
	pub(super) type PlayerCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Holding the tickets that player used to join the pool
	#[pallet::storage]
	pub type Tickets<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Ticket<T::AccountId>>;

	/// Holding the services to serve to players, means service detail can change on runtime
	#[pallet::storage]
	#[pallet::getter(fn services)]
	pub type Services<T: Config> = StorageMap<_, Blake2_128Concat, ID, SystemService>;

	/// The new players join the pool before the TimeService, whose are without charge
	#[pallet::storage]
	pub(super) type NewPlayers<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxPlayerStorage>, ValueQuery>;

	/// Holing players, who stay in the pool longer than TimeService
	#[pallet::storage]
	pub type IngamePlayers<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxPlayerStorage>, ValueQuery>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig {}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<MaxPlayer<T>>::put(<T as Config>::MaxPlayerStorage::get());

			let services = <T as Config>::UpfrontServices::get_default_services();
			for service in services.data {
				Services::<T>::insert(service.0, service.1);
			}
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		PlayerNotFound,
		PlayerCountOverflow,
		ExceedMaxPlayer,
		CanNotClearNewPlayers,
		IntoBalanceFail,
		PoolNotFound,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ChargePoolService,
		UpfrontSetMaxPlayer {
			new_max_player: u32,
		},
		UpfrontSetServiceTXLimit {
			service: ID,
			tx_limit: u32,
		},
		UpfrontSetServiceDiscount {
			service: ID,
			discount: sp_runtime::Permill,
		},
	}

	impl<T: Config> SystemPool<AccountIdLookupOf<T>, T::AccountId> for Pallet<T> {
		/// Join Upfront Pool
		///
		/// The origin must be Signed
		///
		/// Parameters:
		/// - `level`: The level of ticket Basic - Medium - Advance
		///
		/// Weight: `O(1)`
		#[transactional]
		fn join(sender: AccountIdLookupOf<T>, pool_id: ID) -> DispatchResult {
			let sender = T::Lookup::lookup(sender)?;
			let new_player_count =
				Self::player_count().checked_add(1).ok_or(<Error<T>>::PlayerCountOverflow)?;

			ensure!(
				new_player_count <= Self::max_player(),
				<Error<T>>::ExceedMaxPlayer
			);
			{
				let service = Self::get_pool_by_id(pool_id)?;
				let service_fee = u128_try_to_balance::<
					<T as pallet::Config>::Currency,
					T::AccountId,
				>(service.value)?;
				let double_service_fee = service_fee + service_fee;
				ensure!(
					T::Currency::free_balance(&sender) > double_service_fee,
					pallet_balances::Error::<T>::InsufficientBalance
				);
				<NewPlayers<T>>::try_mutate(|newplayers| newplayers.try_push(sender.clone()))
					.map_err(|_| <Error<T>>::ExceedMaxPlayer)?;
				T::Currency::reserve(&sender, service_fee)?;
				T::Currency::withdraw(
					&sender,
					service_fee,
					WithdrawReasons::FEE,
					ExistenceRequirement::KeepAlive,
				)?;
			}
			Self::join_pool(sender, pool_id, new_player_count)?;
			Ok(())
		}

		/// Leave Upfront Pool
		///
		/// The origin must be Signed
		///
		/// Weight: `O(1)`
		#[transactional]
		fn leave(sender: AccountIdLookupOf<T>) -> DispatchResult {
			let sender = T::Lookup::lookup(sender)?;
			if let Some(ticket) = Tickets::<T>::get(sender.clone()) {
				if let TicketType::Upfront(pool_id) = ticket.ticket_type {
					let join_time = ticket.join_time;
					let _now = Self::moment_to_u128(<timestamp::Pallet<T>>::get());

					T::Players::add_time_joined_upfront(
						sender.clone(),
						_now.saturating_sub(join_time),
					);

					let service_fee;
					let charge_fee;
					{
						let service = Self::get_pool_by_id(pool_id)?;
						let refund_fee = Self::get_refund_balance(_now, join_time, service.value);
						charge_fee = u128_try_to_balance::<
							<T as pallet::Config>::Currency,
							T::AccountId,
						>(service.value - refund_fee)?;
						service_fee = u128_try_to_balance::<
							<T as pallet::Config>::Currency,
							T::AccountId,
						>(service.value)?;
					}

					T::Currency::unreserve(&sender, service_fee);
					T::Currency::withdraw(
						&sender,
						charge_fee,
						WithdrawReasons::FEE,
						ExistenceRequirement::KeepAlive,
					)?;

					let new_player_count = Self::player_count()
						.checked_sub(1)
						.ok_or(<Error<T>>::PlayerCountOverflow)?;
					Self::remove_player(&sender, new_player_count);
					return Ok(())
				}
			}
			Err(Error::<T>::PlayerNotFound.into())
		}

		fn get_service(pool_id: ID) -> Option<SystemService> {
			Services::<T>::get(pool_id)
		}

		fn get_ticket(sender: &T::AccountId) -> Option<Ticket<T::AccountId>> {
			match Tickets::<T>::get(sender) {
				Some(data) => Some(data),
				None => None,
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set MaxPlayer
		///
		/// The root must be Signed
		///
		/// Parameters:
		/// - `max_player`: new value of MaxPlayer
		///
		/// Weight: `O(1)`
		#[pallet::weight(0)]
		pub fn set_max_player(origin: OriginFor<T>, max_player: u32) -> DispatchResult {
			ensure_root(origin)?;
			<MaxPlayer<T>>::put(max_player);
			Self::deposit_event(Event::<T>::UpfrontSetMaxPlayer {
				new_max_player: max_player,
			});
			Ok(())
		}

		// Should validate max, min for input
		#[pallet::weight(0)]
		pub fn set_services_tx_limit(
			origin: OriginFor<T>,
			pool_id: ID,
			tx_limit: u32,
		) -> DispatchResult {
			ensure_root(origin)?;
			let mut service_data = Self::get_pool_by_id(pool_id)?;

			service_data.service.tx_limit = tx_limit;
			Services::<T>::insert(pool_id, service_data);

			Self::deposit_event(Event::<T>::UpfrontSetServiceTXLimit {
				service: pool_id,
				tx_limit,
			});

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_services_discount(
			origin: OriginFor<T>,
			pool_id: ID,
			discount: sp_runtime::Permill,
		) -> DispatchResult {
			ensure_root(origin)?;
			let mut service_data = Self::get_pool_by_id(pool_id)?;

			service_data.service.discount = discount;
			Services::<T>::insert(pool_id, service_data);

			Self::deposit_event(Event::<T>::UpfrontSetServiceDiscount {
				service: pool_id,
				discount,
			});

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn join_pool(sender: T::AccountId, pool_id: ID, new_player_count: u32) -> Result<(), Error<T>> {
		let _now = <timestamp::Pallet<T>>::get();
		let ticket = Ticket::<T::AccountId> {
			address: sender.clone(),
			join_time: Self::moment_to_u128(_now),
			ticket_type: TicketType::Upfront(pool_id),
		};
		Tickets::<T>::insert(sender, ticket);
		<PlayerCount<T>>::put(new_player_count);
		Ok(())
	}

	fn move_newplayer_to_ingame() -> Result<(), Error<T>> {
		let new_players: Vec<T::AccountId> = NewPlayers::<T>::get().into_inner();
		for new_player in new_players {
			<IngamePlayers<T>>::try_append(new_player.clone())
				.map_err(|_| <Error<T>>::ExceedMaxPlayer)?;
		}
		<NewPlayers<T>>::kill();
		Ok(())
	}

	fn get_refund_balance(leave_time: u128, join_time: u128, service_fee: u128) -> u128 {
		let period_time = leave_time.saturating_sub(join_time);
		if period_time < T::MasterPool::get_timeservice() {
			service_fee
		} else {
			let extra = period_time % T::MasterPool::get_timeservice();
			service_fee
				.saturating_mul(T::MasterPool::get_timeservice().saturating_sub(extra))
				.saturating_div(T::MasterPool::get_timeservice())
		}
	}

	fn remove_player(player: &T::AccountId, new_player_count: u32) {
		if let Some(pool_id) = Self::get_pool_joined(player) {
			T::MasterPool::remove_player(player, pool_id);
		}

		Tickets::<T>::remove(player);

		<IngamePlayers<T>>::mutate(|players| {
			if let Some(ind) = players.iter().position(|id| id == player) {
				players.swap_remove(ind);
			}
		});

		<NewPlayers<T>>::mutate(|players| {
			if let Some(ind) = players.iter().position(|id| id == player) {
				players.swap_remove(ind);
			}
		});

		<PlayerCount<T>>::put(new_player_count);
	}

	fn charge_ingame() -> Result<(), Error<T>> {
		let ingame_players: Vec<T::AccountId> = IngamePlayers::<T>::get().into_inner();
		for player in ingame_players {
			if let Some(service) = Self::get_player_service(player.clone()) {
				let fee_value =
					u128_to_balance::<<T as pallet::Config>::Currency, T::AccountId>(service.value);

				match T::Currency::withdraw(
					&player,
					fee_value,
					WithdrawReasons::FEE,
					ExistenceRequirement::KeepAlive,
				) {
					Ok(_) => {},
					Err(_) => {
						let new_player_count = Self::player_count()
							.checked_sub(1)
							.ok_or(<Error<T>>::PlayerCountOverflow)?;
						let _ = Self::remove_player(&player, new_player_count);
					},
				};
			}
		}
		Ok(())
	}

	fn get_player_service(player: T::AccountId) -> Option<SystemService> {
		if let Some(pool_id) = Self::get_pool_joined(&player) {
			return Self::get_service(pool_id)
		}
		None
	}

	fn get_pool_joined(player: &T::AccountId) -> Option<ID> {
		if let Some(ticket) = Tickets::<T>::get(player) {
			if let TicketType::Upfront(pool_id) = ticket.ticket_type {
				return Some(pool_id)
			}
		}
		None
	}

	fn moment_to_u128(input: T::Moment) -> u128 {
		sp_runtime::SaturatedConversion::saturated_into(input)
	}

	pub fn get_timestamp() -> u128 {
		let _now: u128 = <timestamp::Pallet<T>>::get().try_into().ok().unwrap_or_default();
		_now
	}

	fn get_pool_by_id(pool_id: ID) -> Result<SystemService, Error<T>> {
		match Services::<T>::get(pool_id) {
			Some(service) => Ok(service),
			//TODO: Change error message
			None => Err(<Error<T>>::PoolNotFound),
		}
	}
}
