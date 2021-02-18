//! Pallet that allows block authors to include a u32 of their choosing. The u32 must be less than
//! the current relay parent block number. This allows the block author to set a mortality for the
//! block in terms of the relay chain itself.
//!
//! NOTE: I don't actually want to use this for parablock mortality. It is actually meant to be a
//! minimum example of "checking this inherent requires data from the parachain inherent".

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_error, decl_module, ensure,
	weights::DispatchClass,
};
use frame_system::ensure_none;
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_inherents::ProvideInherentData;
use sp_inherents::{InherentData, InherentIdentifier, IsFatalError, ProvideInherent};
use sp_runtime::RuntimeString;

pub trait Config: frame_system::Config + cumulus_pallet_parachain_system::Config {}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// This block is not valid (anymore) because the relay parent height exceeds the maximum
		RelayParentTooHigh,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		/// Inherent to set the maximum relay parent height
		#[weight = (
			0,
			DispatchClass::Mandatory
		)]
		fn set_max_relay_parent(origin, max: u32) {
			ensure_none(origin)?;

			ensure!(Self::check_relay_height(max), Error::<T>::RelayParentTooHigh)
		}
	}
}

impl<T: Config> Module<T> {
	/// The actual implementation of checking the inherent. Compares the max relay parent height
	/// from this inherent, to the actual relay parent height from the parachain inherent.
	fn check_relay_height(max: u32) -> bool {

		let maybe_validation_data = cumulus_pallet_parachain_system::Module::<T>::validation_data();
		let relay_height = maybe_validation_data
			.expect("Validation data gets set in parachain system inherent. Parachain system \
					 inherent came before this inherent. Therefore validation data is set. \
					 qed.")
			.relay_parent_number;

		max <= relay_height
	}
}

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"example_";

#[derive(Encode)]
#[cfg_attr(feature = "std", derive(Debug, Decode))]
pub enum InherentError {
	Other(RuntimeString),
}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		match *self {
			InherentError::Other(_) => true,
		}
	}
}

impl InherentError {
	/// Try to create an instance ouf of the given identifier and data.
	#[cfg(feature = "std")]
	pub fn try_from(id: &InherentIdentifier, data: &[u8]) -> Option<Self> {
		if id == &INHERENT_IDENTIFIER {
			<InherentError as parity_scale_codec::Decode>::decode(&mut &data[..]).ok()
		} else {
			None
		}
	}
}

/// The thing that the outer node will use to actually inject the max block number.
/// This one is dead simple and will just include a single fixed max. A more realistic one,
/// would look at the current relay parent height and add an offset, but that isn't necessary
/// to show the idea.
#[cfg(feature = "std")]
pub struct InherentDataProvider(pub u32);

#[cfg(feature = "std")]
impl ProvideInherentData for InherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&INHERENT_IDENTIFIER
	}

	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		inherent_data.put_data(INHERENT_IDENTIFIER, &self.0)
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		InherentError::try_from(&INHERENT_IDENTIFIER, error).map(|e| format!("{:?}", e))
	}
}

impl<T: Config> ProvideInherent for Module<T> {
	type Call = Call<T>;
	type Error = InherentError;
	const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

	fn create_inherent(data: &InherentData) -> Option<Self::Call> {
		let max_relay_height = data
			.get_data::<u32>(&INHERENT_IDENTIFIER)
			.expect("Gets and decodes authorship inherent data")?;

		Some(Call::set_max_relay_parent(max_relay_height))
	}

	fn check_inherent(call: &Self::Call, _data: &InherentData) -> Result<(), Self::Error> {
		// We only care to check our own inherents
		if let Self::Call::set_max_relay_parent(max) = call {
			ensure!(
				Self::check_relay_height(*max),
				InherentError::Other(sp_runtime::RuntimeString::Borrowed("Relay Parent Too High"))
			);
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use frame_support::{
		assert_noop, assert_ok, impl_outer_origin, parameter_types,
		traits::{OnFinalize, OnInitialize},
	};
	use sp_core::H256;
	use sp_io::TestExternalities;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
	};

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		TestExternalities::new(t)
	}

	impl_outer_origin! {
		pub enum Origin for Test where system = frame_system {}
	}

	mod author_inherent {
		pub use super::super::*;
	}

	impl<T> EventHandler<T> for () {
		fn note_author(_author: T) {}
	}

	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
	}
	impl System for Test {
		type BaseCallFilter = ();
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Call = ();
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = ();
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
	}
	impl Config for Test {
		type EventHandler = ();
		type CanAuthor = ();
	}
	type AuthorInherent = Module<Test>;
	type Sys = frame_system::Module<Test>;

	pub fn roll_to(n: u64) {
		while Sys::block_number() < n {
			Sys::on_finalize(Sys::block_number());
			Sys::set_block_number(Sys::block_number() + 1);
			Sys::on_initialize(Sys::block_number());
			AuthorInherent::on_initialize(Sys::block_number());
		}
	}

	#[test]
	fn set_author_works() {
		new_test_ext().execute_with(|| {
			assert_ok!(AuthorInherent::set_author(Origin::none(), 1));
			roll_to(1);
			assert_ok!(AuthorInherent::set_author(Origin::none(), 1));
			roll_to(2);
		});
	}

	#[test]
	fn double_author_fails() {
		new_test_ext().execute_with(|| {
			assert_ok!(AuthorInherent::set_author(Origin::none(), 1));
			assert_noop!(
				AuthorInherent::set_author(Origin::none(), 1),
				Error::<Test>::AuthorAlreadySet
			);
		});
	}
}
