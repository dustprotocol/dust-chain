//! Unit tests for the transaction payment module.

#![cfg(test)]

use super::*;
use frame_support::{
	assert_ok, assert_err,
	weights::{DispatchClass, DispatchInfo, Pays},
};
use mock::{
	AccountId, BlockWeights, Call, Currencies,
	ExtBuilder, Origin, Runtime, TransactionPayment,
	DUST, USDD, ALICE, BOB
};
use orml_traits::MultiCurrency;
use sp_runtime::{testing::TestXt, traits::One};

const CALL: &<Runtime as frame_system::Config>::Call =
	&Call::Currencies(module_currencies::Call::transfer(BOB, USDD, 12));

const CALL2: &<Runtime as frame_system::Config>::Call =
	&Call::Currencies(module_currencies::Call::transfer_native_currency(BOB, 12));

const INFO: DispatchInfo = DispatchInfo {
	weight: 1000,
	class: DispatchClass::Normal,
	pays_fee: Pays::Yes,
};

const POST_INFO: PostDispatchInfo = PostDispatchInfo {
	actual_weight: Some(800),
	pays_fee: Pays::Yes,
};

#[test]
fn charges_fee() {
	ExtBuilder::default().build().execute_with(|| {
		let fee = 23 * 2 + 1000; // len * byte + weight
		assert_eq!(
			ChargeTransactionPayment::<Runtime>::from(0)
				.validate(&ALICE, CALL, &INFO, 23)
				.unwrap()
				.priority,
			fee
		);
		assert_eq!(Currencies::free_balance(DUST, &ALICE), (100000 - fee).into());

		let fee2 = 18 * 2 + 1000; // len * byte + weight
		assert_eq!(
			ChargeTransactionPayment::<Runtime>::from(0)
				.validate(&ALICE, CALL2, &INFO, 18)
				.unwrap()
				.priority,
			fee2
		);
		use sp_runtime::{traits::{UniqueSaturatedInto}};
		assert_eq!(
			Currencies::free_balance(DUST, &ALICE),
			(100000 - fee - fee2).unique_saturated_into()
		);
	});
}

#[test]
fn charges_fee_when_pre_dispatch_and_native_currency_is_enough() {
	ExtBuilder::default().build().execute_with(|| {
		let fee = 23 * 2 + 1000; // len * byte + weight
		assert!(ChargeTransactionPayment::<Runtime>::from(0)
			.pre_dispatch(&ALICE, CALL, &INFO, 23)
			.is_ok());
		assert_eq!(Currencies::free_balance(DUST, &ALICE), 100000 - fee);
	});
}

#[test]
fn refund_fee_according_to_actual_when_post_dispatch_and_native_currency_is_enough() {
	ExtBuilder::default().build().execute_with(|| {
		let fee = 23 * 2 + 1000; // len * byte + weight
		let pre = ChargeTransactionPayment::<Runtime>::from(0)
			.pre_dispatch(&ALICE, CALL, &INFO, 23)
			.unwrap();
		assert_eq!(Currencies::free_balance(DUST, &ALICE), 100000 - fee);

		let refund = 200; // 1000 - 800
		assert!(ChargeTransactionPayment::<Runtime>::post_dispatch(pre, &INFO, &POST_INFO, 23, &Ok(())).is_ok());
		assert_eq!(Currencies::free_balance(DUST, &ALICE), 100000 - fee + refund);
	});
}

#[test]
fn charges_fee_when_validate_and_native_is_not_enough() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(<Currencies as MultiCurrency<_>>::transfer(USDD, &ALICE, &BOB, 1000));
		assert_eq!(<Currencies as MultiCurrency<_>>::free_balance(DUST, &BOB), 0);
		assert_eq!(<Currencies as MultiCurrency<_>>::free_balance(USDD, &BOB), 1000);

		let _fee = 500 * 2 + 1000; // len * byte + weight
		assert_err!(
			ChargeTransactionPayment::<Runtime>::from(0)
				.validate(&BOB, CALL2, &INFO, 500),
				TransactionValidityError::Invalid(InvalidTransaction::Payment)
		);

		// // add liquidity to DEX
		// assert_ok!(DEXModule::add_liquidity(
		// 	Origin::signed(ALICE),
		// 	DUST,
		// 	USDD,
		// 	10000,
		// 	1000,
		// 	false
		// ));
		// assert_eq!(DEXModule::get_liquidity_pool(DUST, USDD), (10000, 1000));

		// let fee = 500 * 2 + 1000; // len * byte + weight
		// assert_eq!(
		// 	ChargeTransactionPayment::<Runtime>::from(0)
		// 		.validate(&BOB, CALL2, &INFO, 500)
		// 		.unwrap()
		// 		.priority,
		// 	fee
		// );


		// assert_eq!(Currencies::free_balance(DUST, &BOB), 0);
		// assert_eq!(Currencies::free_balance(USDD, &BOB), 749);
		// assert_eq!(DEXModule::get_liquidity_pool(DUST, USDD), (10000 - 2000, 1251));
	});
}

#[test]
fn set_default_fee_token_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(TransactionPayment::default_fee_currency_id(&ALICE), None);
		assert_ok!(TransactionPayment::set_default_fee_token(
			Origin::signed(ALICE),
			Some(USDD)
		));
		assert_eq!(TransactionPayment::default_fee_currency_id(&ALICE), Some(USDD));
		assert_ok!(TransactionPayment::set_default_fee_token(Origin::signed(ALICE), None));
		assert_eq!(TransactionPayment::default_fee_currency_id(&ALICE), None);
	});
}


#[test]
fn query_info_works() {
	ExtBuilder::default()
		.base_weight(5)
		.byte_fee(1)
		.weight_fee(2)
		.build()
		.execute_with(|| {
			let call = Call::PalletBalances(pallet_balances::Call::transfer(AccountId::new([2u8; 32]), 69));
			let origin = 111111;
			let extra = ();
			let xt = TestXt::new(call, Some((origin, extra)));
			let info = xt.get_dispatch_info();
			let ext = xt.encode();
			let len = ext.len() as u32;

			// all fees should be x1.5
			NextFeeMultiplier::<Runtime>::put(Multiplier::saturating_from_rational(3, 2));

			assert_eq!(
				TransactionPayment::query_info(xt, len),
				RuntimeDispatchInfo {
					weight: info.weight,
					class: info.class,
					partial_fee: 5 * 2 /* base * weight_fee */
						+ len as u128  /* len * 1 */
						+ info.weight.min(BlockWeights::get().max_block) as u128 * 2 * 3 / 2 /* weight */
				},
			);
		});
}

#[test]
fn compute_fee_works_without_multiplier() {
	ExtBuilder::default()
		.base_weight(100)
		.byte_fee(10)
		.build()
		.execute_with(|| {
			// Next fee multiplier is zero
			assert_eq!(NextFeeMultiplier::<Runtime>::get(), <Multiplier as One>::one());

			// Tip only, no fees works
			let dispatch_info = DispatchInfo {
				weight: 0,
				class: DispatchClass::Operational,
				pays_fee: Pays::No,
			};
			assert_eq!(Pallet::<Runtime>::compute_fee(0, &dispatch_info, 10), 10);
			// No tip, only base fee works
			let dispatch_info = DispatchInfo {
				weight: 0,
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			assert_eq!(Pallet::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);
			// Tip + base fee works
			assert_eq!(Pallet::<Runtime>::compute_fee(0, &dispatch_info, 69), 169);
			// Len (byte fee) + base fee works
			assert_eq!(Pallet::<Runtime>::compute_fee(42, &dispatch_info, 0), 520);
			// Weight fee + base fee works
			let dispatch_info = DispatchInfo {
				weight: 1000,
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			assert_eq!(Pallet::<Runtime>::compute_fee(0, &dispatch_info, 0), 1100);
		});
}

#[test]
fn compute_fee_works_with_multiplier() {
	ExtBuilder::default()
		.base_weight(100)
		.byte_fee(10)
		.build()
		.execute_with(|| {
			// Add a next fee multiplier. Fees will be x3/2.
			NextFeeMultiplier::<Runtime>::put(Multiplier::saturating_from_rational(3, 2));
			// Base fee is unaffected by multiplier
			let dispatch_info = DispatchInfo {
				weight: 0,
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			assert_eq!(Pallet::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);

			// Everything works together :)
			let dispatch_info = DispatchInfo {
				weight: 123,
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			// 123 weight, 456 length, 100 base
			assert_eq!(
				Pallet::<Runtime>::compute_fee(456, &dispatch_info, 789),
				100 + (3 * 123 / 2) + 4560 + 789,
			);
		});
}

#[test]
fn compute_fee_works_with_negative_multiplier() {
	ExtBuilder::default()
		.base_weight(100)
		.byte_fee(10)
		.build()
		.execute_with(|| {
			// Add a next fee multiplier. All fees will be x1/2.
			NextFeeMultiplier::<Runtime>::put(Multiplier::saturating_from_rational(1, 2));

			// Base fee is unaffected by multiplier.
			let dispatch_info = DispatchInfo {
				weight: 0,
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			assert_eq!(Pallet::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);

			// Everything works together.
			let dispatch_info = DispatchInfo {
				weight: 123,
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			// 123 weight, 456 length, 100 base
			assert_eq!(
				Pallet::<Runtime>::compute_fee(456, &dispatch_info, 789),
				100 + (123 / 2) + 4560 + 789,
			);
		});
}

#[test]
fn compute_fee_does_not_overflow() {
	ExtBuilder::default()
		.base_weight(100)
		.byte_fee(10)
		.build()
		.execute_with(|| {
			// Overflow is handled
			let dispatch_info = DispatchInfo {
				weight: Weight::max_value(),
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			assert_eq!(
				Pallet::<Runtime>::compute_fee(<u32>::max_value(), &dispatch_info, <u128>::max_value()),
				<u128>::max_value()
			);
		});
}
