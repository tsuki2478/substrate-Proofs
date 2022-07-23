#![cfg_attr(not(feature ="std"),no_std)]

///  A module for proof of existence
pub use pallet::*;

/// 定义具体的一个功能模块
#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	// 定义配置接口
	#[pallet::config]
	pub trait Config:frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// 定义相关存储类型，且自带方法
    #[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub type Proofs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Vec<u8>,
        (T::AccountId, T::BlockNumber)
    >;

	// 定义触发
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")] // 转义，让客户端可以识别
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T:Config> {
		ClaimCreated(T::AccountId,Vec<u8>),
		ClaimRevoked(T::AccountId,Vec<u8>),
		ClaimMoved(T::AccountId, T::AccountId, Vec<u8>),
	}

	// 定义错误类型
	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimNotExist,
		NotClaimOwner,
		DestinationIsClaimOwner
	}

	// 出块前后的回调
	#[pallet::hooks]
	impl<T:Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
	}

	// 定义具体逻辑处理方法
	#[pallet::call]
	impl<T:Config>Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_claim(
			origin:OriginFor<T>,
			claim:Vec<u8>
		)-> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?; // 校验发送方是已经签名的校验

			ensure!(!Proofs::<T>::contains_key(&claim),Error::<T>::ProofAlreadyExist); //校验

			Proofs::<T>::insert(
				&claim,
				(sender.clone(), frame_system::Pallet::<T>::block_number())
			);

			Self::deposit_event(Event::ClaimCreated(sender,claim));
			Ok(().into()) // 返回结果，result类型并且转换
		}
		// 权重
		#[pallet::weight(0)]
		pub fn revoke_claim(
			origin:OriginFor<T>,
			claim:Vec<u8>
		)->DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?; // 校验发送方是已经签名的校验，获取AccountID sender

			let (owner,_) =Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?; // 校验是否存在这个值，不存在则没必要继续

			ensure!(owner == sender, Error::<T>::NotClaimOwner); // 校验当前交易的发送方是否是 owner

			Proofs::<T>::remove(&claim);

			Self::deposit_event(Event::ClaimRevoked(sender,claim));
			Ok(().into()) // 返回结果，result类型并且转换
		}

		       #[pallet::weight(0)]
        pub fn move_claim(
            origin: OriginFor<T>,
            destination: T::AccountId,
            claim: Vec<u8>
        ) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?; // 校验发送方是已经签名的校验，获取AccountID sender

			let (owner,_) =Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?; // 校验是否存在这个值，不存在则没必要继续

			ensure!(owner == sender, Error::<T>::NotClaimOwner); // 校验当前交易的发送方是否是 owner

            ensure!(owner != destination, Error::<T>::DestinationIsClaimOwner);

			Proofs::<T>::remove(&claim);

			Proofs::<T>::insert(
                &claim,
                (destination.clone(), <frame_system::Pallet::<T>>::block_number()),
            );

            Self::deposit_event(Event::ClaimMoved(sender, destination, claim));
            Ok(().into())
        }
	}


}

