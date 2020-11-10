#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;

#[ink::contract]
mod charityraffle {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::collections::HashMap as StorageHashMap;
    use ink_storage::collections::Vec;

    const MINIMUM_TOKEN: u128 = 10_000_000_000_000; // 0.01 unit
    const MAXIMUM_TOKEN: u128 = 100_000_000_000_000; // 0.1 unit

    const COUNTDOWN: Timestamp = 60000; // 60s or 900000

    #[ink(storage)]
    pub struct Charity {
        beneficiary: AccountId,
        players: StorageHashMap<AccountId, Balance>,
        players_vec: Vec<AccountId>,
        countdown: Option<Timestamp>,
        winners: Vec<AccountId>,
        amount_collected: Balance,
    }

    #[ink(event)]
    pub struct Play {
        player: AccountId,
    }

    #[ink(event)]
    pub struct Draw {
        winner: AccountId,
    }

    #[ink(event)]
    pub struct CountDownStarted {
        player: AccountId,
        timestamp: Timestamp,
    }

    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    pub enum Error {
        Finished,
        InvalidEntryAmount,
        HasPlayed,
        CountDownNotStarted,
        NotEnoughPlayer,
    }

    pub type Re<T> = core::result::Result<T, Error>;
    pub use Error::*;

    impl Charity {
        #[ink(constructor)]
        pub fn new(beneficiary: AccountId) -> Self {
            Self {
                beneficiary,
                players: StorageHashMap::new(),
                players_vec: Vec::new(),
                winners: Vec::new(),
                countdown: None,
                amount_collected: 0,
            }
        }

        #[ink(message)]
        pub fn get_beneficiary_id(&self) -> AccountId {
            self.beneficiary
        }

        #[ink(message)]
        pub fn participants(&self) -> u32 {
            self.players_vec.len()
        }

        #[ink(message)]
        pub fn finished(&self) -> bool {
            return self.winners.len() == 2;
        }

        #[ink(message)]
        pub fn winners_address(&self) -> (Option<AccountId>, Option<AccountId>) {
            let first = self.winners.first().map(|w| w.clone());
            let last = self.winners.last().map(|w| w.clone());
            return (first, last);
        }

        #[ink(message)]
        pub fn get_amount_collected(&self) -> Balance {
            self.amount_collected
        }

        #[ink(message, payable)]
        pub fn play(&mut self) -> Re<()> {
            if self.finished() {
                return Err(Finished);
            }

            let caller = self.env().caller();
            let amount = self.env().transferred_balance();

            if amount >= MAXIMUM_TOKEN || amount <= MINIMUM_TOKEN {
                return Err(InvalidEntryAmount);
            }

            if self.players.contains_key(&caller) {
                return Err(HasPlayed);
            }

            self.players.insert(caller, amount);
            self.players_vec.push(caller);
            self.amount_collected += amount;

            self.env().emit_event(Play { player: caller });

            if self.countdown.is_none() && self.participants() >= 5 {
                let timestamp = Self::env().block_timestamp();
                self.countdown = Some(timestamp);

                self.env().emit_event(CountDownStarted {
                    timestamp,
                    player: caller,
                });
            }
            Ok(())
        }

        #[ink(message)]
        pub fn draw(&mut self) -> Re<()> {
            if self.finished() {
                return Err(Finished);
            }

            if self.countdown.is_some()
                && Self::env().block_timestamp() - self.countdown.unwrap() < COUNTDOWN
            {
                return Err(CountDownNotStarted);
            }

            if self.participants() < 5 {
                return Err(NotEnoughPlayer);
            }

            let choosed_winner = Self::get_random_number() % self.participants();
            let winner = self.players_vec[choosed_winner];
            self.winners.push(winner);

            self.env().emit_event(Draw { winner });

            let last_player = self.players_vec.pop().unwrap();
            let _ = self.players_vec.set(choosed_winner, last_player);

            Ok(())
        }

        fn get_random_number() -> u32 {
            let seed: [u8; 8] = [1, 1, 1, 1, 1, 1, 1, 1];
            let random_hash = Self::env().random(&seed);
            Self::as_u32_be(&random_hash.as_ref())
        }

        fn as_u32_be(arr: &[u8]) -> u32 {
            ((arr[0] as u32) << 24)
                + ((arr[1] as u32) << 16)
                + ((arr[2] as u32) << 8)
                + ((arr[3] as u32) << 0)
        }
    }
}
