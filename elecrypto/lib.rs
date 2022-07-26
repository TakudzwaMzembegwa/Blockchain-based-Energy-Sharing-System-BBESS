#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod elecrypto {
    use ink_storage::traits::SpreadAllocate;

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    #[cfg(not(feature = "ink-as-dependency"))]
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Elecrypto {
        /// The total supply of the token.
        total_supply: Balance,
        /// The balance of each user.
        balances: ink_storage::Mapping<AccountId, Balance>,
        /// Approval spender on behalf of the message's sender.
        allowances: ink_storage::Mapping<(AccountId, AccountId), Balance>,
    }

    impl Elecrypto {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.total_supply = initial_supply;
                let caller = Self::env().caller();
                contract.balances.insert(&caller, &initial_supply);

                Self::env().emit_event(Transfer {
                    from: None,
                    to: Some(caller),
                    value: initial_supply,
                });
            })
        }
        // Return function of total supply
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        // Return function of balance for a specific account
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(&owner).unwrap_or_default()
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> bool {
            // Record the new allowance.
            let owner = self.env().caller();
            self.allowances.insert(&(owner, spender), &value);

            // Notify offchain users of the approval and report success.
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });

            true
        }

        #[ink(message)]
        pub fn allowancetoken(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_of_or_zero(&owner, &spender)
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> bool {
            // Ensure that a sufficient allowance exists.
            let caller = self.env().caller();
            let allowance = self.allowance_of_or_zero(&from, &caller);
            if allowance < value {
                return false;
            }

            let transfer_result = self.transfer_from_to(from, to, value);
            if !transfer_result {
                return false;
            }

            // Deduct the value of the allowance token and transfer the tokens.
            self.allowances.insert((from, caller), &(allowance - value));
            true
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> bool {
            self.transfer_from_to(self.env().caller(), to, value)
        }

        fn transfer_from_to(&mut self, from: AccountId, to: AccountId, value: Balance) -> bool {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return false;
            }

            // Update the sender's balance.
            self.balances.insert(&from, &(from_balance - value));

            // Update the receiver's balance.
            let to_balance = self.balance_of(to);
            self.balances.insert(&to, &(to_balance + value));

            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });

            true
        }

        fn allowance_of_or_zero(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            // To read more: https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
            self.allowances.get(&(*owner, *spender)).unwrap_or_default()
        }
    }

    // Performing units test
    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn new_works() {
            let contract = Elecrypto::new(7000000);
            assert_eq!(contract.total_supply(), 7000000);
        }

        #[ink::test]
        fn balance_works() {
            let contract = Elecrypto::new(23000);
            assert_eq!(contract.total_supply(), 23000);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 23000);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = Elecrypto::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert!(contract.transfer(AccountId::from([0x0; 32]), 10));
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
            assert!(!contract.transfer(AccountId::from([0x0; 32]), 100));
        }

        #[ink::test]
        fn transfer_from_works() {
            let mut contract = Elecrypto::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            contract.approve(AccountId::from([0x1; 32]), 20);
            contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 10);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
        }

        #[ink::test]
        fn allowances_works() {
            let mut contract = Elecrypto::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            contract.approve(AccountId::from([0x1; 32]), 200);
            assert_eq!(
                contract.allowancetoken(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])),
                200
            );

            assert!(contract.transfer_from(
                AccountId::from([0x1; 32]),
                AccountId::from([0x0; 32]),
                50
            ));
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 50);
            assert_eq!(
                contract.allowancetoken(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])),
                150
            );

            assert!(!contract.transfer_from(
                AccountId::from([0x1; 32]),
                AccountId::from([0x0; 32]),
                100
            ));
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 50);
            assert_eq!(
                contract.allowancetoken(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])),
                150
            );
        }
    }
}
