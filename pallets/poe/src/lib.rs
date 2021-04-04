#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
// 添加ensure, StorageMap, 没删dispatch, traits::Get
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure};
use frame_system::ensure_signed;
use sp_runtime::{traits::StaticLookup};
// 添加Vec
use sp_std::vec::Vec;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
// pub trait Trait: frame_system::Trait {
// 	/// Because this pallet emits events, it depends on the runtime's definition of an event.
// 	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
// }

/// 这个Config应该是trait
pub trait Config: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Config> as Poe {
		Proofs: map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
// 这里面Trait是config
decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Trait>::AccountId,
	{
		// proof新建, 发出event, [who, claim]
		ClaimCreated(AccountId, Vec<u8>),
		// proof删除, 发出event [who, claim]]
		ClaimRevoked(AccountId, Vec<u8>),
		// proof转移所有权, 发出event,
		ClaimTransfered(AccountId, AccountId, Vec<u8>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// proof已经被创建
		ProofAlreadyClaimed,
		/// 删除不存在的proof
		NoSuchProof,
		/// 删除别人的proof
		NotProofOwner,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000]
		fn create_claim(origin, proof: Vec<u8>) {
			// 检查是否被签名
			let sender = ensure_signed(origin)?;
			// 确认是否被声明过
			ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);
			// get 当前block
			let current_block = <frame_system::Module<T>>::block_number();
			// 插入proof
			Proofs::<T>::insert(&proof, (&sender, current_block));
			// emit event
			Self::deposit_event(RawEvent::ClaimCreated(sender, proof));
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000]
		fn revoke_claim(origin, proof: Vec<u8>) {
			let sender = ensure_signed(origin)?;
			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);
			let (owner, _) = Proofs::<T>::get(&proof);
			ensure!(sender == owner, Error::<T>::NotProofOwner);
			Proofs::<T>::remove(&proof);
			Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));
		}

		#[weight = 10_000]
		fn transfer_claim(origin, dest: <T::Lookup as StaticLookup>::Source, proof: Vec<u8>) {
			let sender = ensure_signed(origin)?;
			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);
			let (owner, _) = Proofs::<T>::get(&proof);
			ensure!(sender == owner, Error::<T>::NotProofOwner);
			let current_block = <frame_system::Module<T>>::block_number();
			let receiver = T::Lookup::lookup(dest)?;
			Proofs::<T>::mutate(&proof, |val| {
				*val = (receiver.clone(), current_block);
			});
			Self::deposit_event(RawEvent::ClaimTransfered(sender, receiver, proof));
		}
	}
}
