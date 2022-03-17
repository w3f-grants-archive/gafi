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
	use frame_support::dispatch::{DispatchResult, Vec};
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{
		tokens::{ExistenceRequirement, WithdrawReasons},
		Currency, Randomness,
	};
	use frame_system::pallet_prelude::*;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Struct, Enum
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Player<T: Config> {
		address: T::AccountId,
		lucky_number: u8,
	}
	impl<T: Config> MaxEncodedLen for Player<T> {
		fn max_encoded_len() -> usize {
			1000
		}
	}
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		type GameRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Storage
	#[pallet::storage]
	#[pallet::getter(fn players)]
	pub(super) type Players<T: Config> = StorageMap<_, Twox64Concat, u32, Player<T>>;

	#[pallet::storage]
	#[pallet::getter(fn game_reward)]
	pub(super) type GameReward<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn players_count)]
	pub(super) type PlayerCount<T: Config> = StorageValue<_, u32, ValueQuery>;

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
		EndGame(u8),
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
			let who_clone = who.clone();
			Self::change_fee(&who, <TicketPrice<T>>::try_get().unwrap())?;

			let count = match <PlayerCount<T>>::try_get() {
				Ok(_) => <PlayerCount<T>>::get(),
				Err(_) => 0,
			};
			let new_player: Player<T> = Player::<T> { address: who, lucky_number };

			<Players<T>>::insert(count + 1, new_player);
			<PlayerCount<T>>::put(count + 1);

			let game_reward = match <GameReward<T>>::try_get() {
				Ok(_) => <GameReward<T>>::try_get().unwrap(),
				Err(_) => Self::u64_to_balance(0).unwrap(),
			};

			// Update storage.
			// <Something<T>>::put(something);
			<GameReward<T>>::put(game_reward + <TicketPrice<T>>::try_get().unwrap());

			// Emit an event.
			Self::deposit_event(Event::PlayerJoin(
				who_clone,
				lucky_number,
				<TicketPrice<T>>::try_get().unwrap(),
			));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn join_all(origin: OriginFor<T>) -> DispatchResult {
			for number in 0..=255 {
				let o = origin.clone();
				Self::join_game(o, number);
			}
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn set_ticket_price(origin: OriginFor<T>, price: BalanceOf<T>) -> DispatchResult {
			<TicketPrice<T>>::put(price);
			let who = ensure_signed(origin)?;
			Self::deposit_event(Event::SetTicketPrice(who, price));
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn end_game(origin: OriginFor<T>) -> DispatchResult {
			Self::finish_game();
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
		// pub fn reset() {
		// 	PlayerCount::put(0);
		// 	<GameReward<T>>::put(Self::u64_to_balance(0).unwrap());
		// 	// <Players<T>>::
		// }
		pub fn balance_to_u64(input: BalanceOf<T>) -> Option<u64> {
			TryInto::<u64>::try_into(input).ok()
		}

		pub fn u64_to_balance(input: u64) -> Option<BalanceOf<T>> {
			input.try_into().ok()
		}

		pub fn u64_to_block(input: u64) -> Option<T::BlockNumber> {
			input.try_into().ok()
		}

		pub fn rand() -> Result<u8, Error<T>> {
			let payload =
				(T::GameRandomness::random(&b""[..]).0, <frame_system::Pallet<T>>::block_number());
			Ok(payload.0.as_ref()[0] as u8)
		}

		pub fn finish_game() -> DispatchResult {
			//choose lucky number
			let lucky = Self::rand().unwrap();
			//get winners
			let mut winners = Vec::new();
			for n in 1..(<PlayerCount<T>>::get() + 1) {
				match <Players<T>>::try_get(n) {
					Ok(_) => {
						let player = <Players<T>>::try_get(n).unwrap();
						if player.lucky_number == lucky {
							winners.push(player)
						};
					},
					Err(_) => (),
				}
			}
			//send reward
			let winners_count = winners.len();
			if winners_count > 0 {
				let total_reward = TryInto::<usize>::try_into(<GameReward<T>>::get()).ok().unwrap();
				let reward = total_reward / winners_count;
				for p in winners {
					T::Currency::deposit_into_existing(&p.address, reward.try_into().ok().unwrap());
				}
				//reset reward
				<GameReward<T>>::put(Self::u64_to_balance(0).unwrap());
			}
			//reset players
			<PlayerCount<T>>::put(0);
			//emit event
			Self::deposit_event(Event::EndGame(lucky));
			Ok(())
		}
	}
}
