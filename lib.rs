#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod raffle {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::collections::HashMap as StorageHashMap;

    use ink_env::hash::Keccak256;
    const MIN_PARTICIPANTS: u64 = 5;
    const MAX_WINNERS: u64 = 2;
    const COUNTDOWN: u64 = 900000;
    const MIN_TOKEN: u128 = 10_000_000_000_000; // 0.01 unit
    const MAX_TOKEN: u128 = 100_000_000_000_000; // 0.1 unit

    #[ink(storage)]
    pub struct Raffle {
        beneficiary: AccountId,
        amount_collected: u128,
        participants: u64,
        draws: u64,
        start_time: Timestamp,
        end_time: Timestamp,
        players: StorageHashMap<AccountId, u64>,
        entries: StorageHashMap<u64, AccountId>,
        winners: [Option<AccountId>; MAX_WINNERS as usize],
    }

    impl Raffle {
        #[ink(constructor)]
        pub fn new(beneficiary: AccountId) -> Self {
            Self {
                beneficiary: beneficiary,
                amount_collected: 0,
                participants: 0,
                draws: 0,
                start_time: 0,
                end_time: 0,
                players: StorageHashMap::new(),
                entries: StorageHashMap::new(),
                winners: [None, None],
            }
        }

        fn now(&self) -> Timestamp {
            return self.env().block_timestamp();
        }

        fn rand(&self) -> u64 {
            let mut output: u64 = 0;
            let encodable = [
                self.now(),
                self.start_time,
                self.end_time,
                self.participants,
                self.draws,
            ];
            let encoded = self.env().hash_encoded::<Keccak256, _>(&encodable);
            let mut hashed = self.env().random(&encoded);
            let random = hashed.as_mut();
            for rand in random.iter() {
                output += *rand as u64;
            }
            return output;
        }

        #[ink(message, payable)]
        pub fn play(&mut self) {
            let now = self.now();
            let caller = self.env().caller();
            let amount = self.env().transferred_balance();
            assert!(
                self.end_time == 0 || self.participants < MIN_PARTICIPANTS || now < self.end_time,
                "Closed for new entants"
            );

            assert!(
                amount >= MIN_TOKEN && amount <= MAX_TOKEN,
                "Wrong amount paid"
            );

            assert!(
                self.players.contains_key(&caller) == false,
                "Must only enter once"
            );

            self.participants += 1;
            self.players.insert(caller, self.participants);
            self.entries.insert(self.participants, caller);

            if self.participants == MIN_PARTICIPANTS {
                self.start_time = self.now();
                self.end_time = self.start_time + COUNTDOWN;
            }

            self.amount_collected += amount;
        }

        #[ink(message)]
        pub fn draw(&mut self) {
            assert!(
                self.end_time > 0
                    && self.participants >= MIN_PARTICIPANTS
                    && self.now() >= self.end_time,
                "Not ready to draw yet"
            );

            assert!(self.draws < MAX_WINNERS, "Winners already decided");
            let winner = self.rand() % self.participants + 1;

            let winning_account = self.entries[&winner];
            self.winners[self.draws as usize] = Some(winning_account);

            self.draws += 1;

            if self.draws == MAX_WINNERS {
                let _ = self.env().transfer(self.beneficiary, self.amount_collected);
            }
        }

        #[ink(message)]
        pub fn get_start(&self) -> u64 {
            return self.start_time;
        }

        #[ink(message)]
        pub fn get_beneficiary_id(&self) -> AccountId {
            return self.beneficiary;
        }

        #[ink(message)]
        pub fn get_end(&self) -> u64 {
            return self.end_time;
        }

        #[ink(message)]
        pub fn participants_counter(&self) -> u64 {
            return self.participants;
        }

        #[ink(message)]
        pub fn get_winners_drawn(&self) -> u64 {
            return self.draws;
        }

        #[ink(message)]
        pub fn get_winners(&self) -> [Option<AccountId>; MAX_WINNERS as usize] {
            return self.winners;
        }

        #[ink(message)]
        pub fn get_amount_collected(&self) -> u128 {
            return self.amount_collected;
        }
    }
}
