#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::dispatch::DispatchResult;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{
		tokens::{ExistenceRequirement, WithdrawReasons},
		Currency,
	};
	use frame_system::pallet_prelude::*;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Storage
	#[pallet::storage]
	#[pallet::getter(fn players)]
	pub(super) type Players<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u8>;

	#[pallet::storage]
	#[pallet::getter(fn game_reward)]
	pub(super) type GameReward<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn ticket_price)]
	pub(super) type TicketPrice<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		// SomethingStored(u32, T::AccountId),
		PlayerJoin(T::AccountId, u8, BalanceOf<T>),
		EndGame(u8, BalanceOf<T>),
		SetTicketPrice(T::AccountId, BalanceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(1000)]
		pub fn join_game(origin: OriginFor<T>, lucky_number: u8) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;
			Self::change_fee(&who, <TicketPrice<T>>::try_get().unwrap())?;
			<Players<T>>::insert(&who, lucky_number);

			let game_reward = match <GameReward<T>>::try_get() {
				Ok(_) => <GameReward<T>>::try_get().unwrap(),
				Err(_) => Self::u64_to_balance(0).unwrap(),
			};
			// Update storage.
			// <Something<T>>::put(something);
			<GameReward<T>>::put(game_reward + <TicketPrice<T>>::try_get().unwrap());
			// Emit an event.
			Self::deposit_event(Event::PlayerJoin(
				who,
				lucky_number,
				<TicketPrice<T>>::try_get().unwrap(),
			));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn set_ticket_price(origin: OriginFor<T>, price: BalanceOf<T>) -> DispatchResult {
			<TicketPrice<T>>::put(price);
			let who = ensure_signed(origin)?;
			Self::deposit_event(Event::SetTicketPrice(who, price));
			Ok(())
		}
	}
	impl<T: Config> Pallet<T> {
		pub fn change_fee(sender: &T::AccountId, fee: BalanceOf<T>) -> DispatchResult {
			let withdraw = T::Currency::withdraw(
				&sender,
				fee,
				WithdrawReasons::RESERVE,
				ExistenceRequirement::KeepAlive,
			);

			match withdraw {
				Ok(_) => Ok(()),
				Err(err) => Err(err),
			}
		}

		pub fn balance_to_u64(input: BalanceOf<T>) -> Option<u64> {
			TryInto::<u64>::try_into(input).ok()
		}

		pub fn u64_to_balance(input: u64) -> Option<BalanceOf<T>> {
			input.try_into().ok()
		}

		pub fn u64_to_block(input: u64) -> Option<T::BlockNumber> {
			input.try_into().ok()
		}
	}
}
