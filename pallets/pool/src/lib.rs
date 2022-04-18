#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use crate::weights::WeightInfo;
use gafi_primitives::pool::{GafiPool, PlayerTicket, Service, MasterPool, TicketType};
use frame_support::traits::Currency;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;

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
	use super::*;
	use frame_support::{pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type UpfrontPool: GafiPool<Self::AccountId>;
		type StakingPool: GafiPool<Self::AccountId>;
		type WeightInfo: WeightInfo;
		// type SponsoredPool: GafiPool<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub struct TicketInfo {
		pub ticket_type: TicketType,
		pub ticket_remain: u32,
	}

	impl TicketInfo {
		pub fn withdraw_ticket(&self) -> Option<Self> {
			if let Some(new_ticket_remain) = self.ticket_remain.checked_sub(1) {
				return Some(TicketInfo {
					ticket_remain: new_ticket_remain,
					ticket_type: self.ticket_type,
				});
			}
			None
		}
	}


	#[pallet::storage]
	pub(super) type Tickets<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, TicketInfo>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Joined { sender: T::AccountId, ticket: TicketType },
		Leaved { sender: T::AccountId, ticket: TicketType },
	}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyJoined,
		NotFoundInPool,
		ComingSoon,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as pallet::Config>::WeightInfo::join(100u32, *ticket))]
		pub fn join(origin: OriginFor<T>, ticket: TicketType) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Tickets::<T>::get(sender.clone()) == None, <Error<T>>::AlreadyJoined);

			match ticket {
				TicketType::Upfront(level) => T::UpfrontPool::join(sender.clone(), level)?,
				TicketType::Staking(level) => T::StakingPool::join(sender.clone(), level)?,
				TicketType::Sponsored(_) => {
					return Err(Error::<T>::ComingSoon.into());
				},
			}

			let service = Self::get_service(ticket);

			let ticket_info = TicketInfo {
				ticket_type: ticket,
				ticket_remain: service.tx_limit,
			};

			Tickets::<T>::insert(sender.clone(), ticket_info);
			Self::deposit_event(Event::<T>::Joined { sender, ticket });
			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::leave(100u32))]
		pub fn leave(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			if let Some(ticket) = Tickets::<T>::get(sender.clone()) {
				match ticket.ticket_type {
					TicketType::Upfront(_) => T::UpfrontPool::leave(sender.clone())?,
					TicketType::Staking(_) => T::StakingPool::leave(sender.clone())?,
					TicketType::Sponsored(_) => {
						return Err(Error::<T>::ComingSoon.into());
					},
				}
				Tickets::<T>::remove(sender.clone());
				Self::deposit_event(Event::<T>::Leaved { sender: sender, ticket: ticket.ticket_type});
				Ok(())
			} else {
				return Err(Error::<T>::NotFoundInPool.into());
			}
		}
	}

	impl<T: Config> PlayerTicket<T::AccountId> for Pallet<T> {
		fn use_ticket(player: T::AccountId) -> Option<TicketType> {
			if let Some(ticket_info) = Tickets::<T>::get(player.clone()) {
				if let Some(new_ticket_info) = ticket_info.withdraw_ticket() {
					Tickets::<T>::insert(player, new_ticket_info);
					return Some(new_ticket_info.ticket_type);
				}
			}
			None
		}

		fn get_service(ticket: TicketType) -> Service {
			match ticket {
				TicketType::Upfront(level) => T::UpfrontPool::get_service(level),
				TicketType::Staking(level) => T::StakingPool::get_service(level),
				TicketType::Sponsored(_) => todo!(),
			}
		}
	}

	impl<T: Config> MasterPool<T::AccountId> for Pallet<T> {
		fn remove_player(player: &T::AccountId) {
			Tickets::<T>::remove(&player);
		}
	}
}
