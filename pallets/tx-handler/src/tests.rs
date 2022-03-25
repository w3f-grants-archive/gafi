use crate::{mock::*, Error, Mapping, Config};
use frame_support::{assert_err, assert_ok, traits::Currency};
use pallet_evm::{AddressMapping};
use hex_literal::hex;
use sp_core::H160;
use sp_runtime::AccountId32;
use std::str::FromStr;

#[test]
fn verify_owner_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		let signature: [u8; 65] = hex!("2bda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		assert_eq!(PalletTxHandler::verify_bind(ALICE, signature, address.to_fixed_bytes()), true, "verify should works");
	});
}

#[test]
fn bind_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();

		let signature: [u8; 65] = hex!("2bda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		assert_ok!(PalletTxHandler::bind(Origin::signed(ALICE.clone()), signature, address, true));

		let account_id = Mapping::<Test>::get(address).unwrap();
		assert_eq!(account_id, ALICE, "AccountId not correct");
	});
}

#[test]
fn bind_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		// incorrect address
		{
			run_to_block(10);
			let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
			let signature: [u8; 65] = hex!("2bda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
			let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84c").unwrap(); //incorrect address

			assert_err!(
				PalletTxHandler::bind(Origin::signed(ALICE), signature, address, true),
				<Error<Test>>::SignatureOrAddressNotCorrect
			);
		}

		// incorrect sender
		{
			run_to_block(10);
		let BOB = AccountId32::from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap();
		let signature: [u8; 65] = hex!("2bda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();

		assert_err!(
			PalletTxHandler::bind(Origin::signed(BOB), signature, address, true),
			<Error<Test>>::SignatureOrAddressNotCorrect
		);
		}

		// incorrect signature
		{
			run_to_block(10);
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();

		let signature: [u8; 65] = hex!("2cda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		assert_err!(
			PalletTxHandler::bind(Origin::signed(ALICE), signature, address, true),
			<Error<Test>>::SignatureOrAddressNotCorrect
		);
		}

		// account already bind
		{
			run_to_block(10);
			let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
	
			let signature: [u8; 65] = hex!("2bda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
			let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
	
			assert_ok!(PalletTxHandler::bind(
				Origin::signed(ALICE.clone()),
				signature,
				address,
				true
			));

			assert_err!(
				PalletTxHandler::bind(Origin::signed(ALICE.clone()), signature, address, true),
				<Error<Test>>::AccountAlreadyBind
			);
		}
	})
}


#[test]
fn transfer_all_works() {
		ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		const ALICE_BALANCE: u64 = 1_000_000_000;
		let signature: [u8; 65] = hex!("2bda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		{
			let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, ALICE_BALANCE);
			assert_eq!(Balances::free_balance(&ALICE), ALICE_BALANCE);
		}
		
		let evm_address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		assert_ok!(PalletTxHandler::bind(Origin::signed(ALICE.clone()), signature, evm_address, false));

		if let Some(mapping_address) = <Test as Config>::AddressMapping::into_account_id(evm_address) {

		PalletTxHandler::transfer_all(evm_address, ALICE.clone());
		// evm_address balance should  
		{
			assert_eq!(Balances::free_balance(&mapping_address), ALICE_BALANCE);
			assert_eq!(Balances::free_balance(&ALICE), ALICE_BALANCE);
		}
		}
	});
}

