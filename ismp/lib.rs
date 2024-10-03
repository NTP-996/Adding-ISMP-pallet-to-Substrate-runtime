#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn messages)]
    pub type Messages<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u8>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        MessageStored(T::AccountId, Vec<u8>),
        MessageSent(T::AccountId, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        MessageTooLong,
        NoMessageFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn store_message(origin: OriginFor<T>, message: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(message.len() <= 256, Error::<T>::MessageTooLong);

            <Messages<T>>::insert(&sender, message.clone());

            Self::deposit_event(Event::MessageStored(sender, message));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads(1) + T::DbWeight::get().writes(1))]
        pub fn send_message(origin: OriginFor<T>, recipient: T::AccountId) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let message = <Messages<T>>::get(&sender);
            ensure!(!message.is_empty(), Error::<T>::NoMessageFound);

            <Messages<T>>::insert(&recipient, message.clone());
            <Messages<T>>::remove(&sender);

            Self::deposit_event(Event::MessageSent(sender, message));
            Ok(().into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as pallet_ismp;
    use frame_support::{assert_ok, construct_runtime, parameter_types};
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };

    type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
    type Block = frame_system::mocking::MockBlock<Test>;

    construct_runtime!(
        pub enum Test where
            Block = Block,
            NodeBlock = Block,
            UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
            ISMPModule: pallet_ismp::{Pallet, Call, Storage, Event<T>},
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub BlockWeights: frame_system::limits::BlockWeights =
            frame_system::limits::BlockWeights::simple_max(1024);
    }

    impl frame_system::Config for Test {
        type BaseCallFilter = ();
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Call = Call;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = Event;
        type BlockHashCount = BlockHashCount;
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ();
        type OnSetCode = ();
    }

    impl Config for Test {
        type Event = Event;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
        t.into()
    }

    #[test]
    fn store_message_works() {
        new_test_ext().execute_with(|| {
            let message = vec![1, 2, 3, 4];
            assert_ok!(ISMPModule::store_message(Origin::signed(1), message.clone()));
            assert_eq!(ISMPModule::messages(1), message);
        });
    }

    #[test]
    fn send_message_works() {
        new_test_ext().execute_with(|| {
            let message = vec![1, 2, 3, 4];
            assert_ok!(ISMPModule::store_message(Origin::signed(1), message.clone()));
            assert_ok!(ISMPModule::send_message(Origin::signed(1), 2));
            assert_eq!(ISMPModule::messages(2), message);
            assert_eq!(ISMPModule::messages(1), Vec::<u8>::new());
        });
    }
}

