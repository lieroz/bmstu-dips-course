use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    owner: String,
    balance: u64,
    currency: String,
}

impl Wallet {
    pub fn new(owner: String, balance: u64, currency: String) -> Wallet {
        Wallet{
            owner,
            balance,
            currency,
        }
    }
    pub fn get_owner(&self) -> String {
        self.owner.clone()
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub fn get_currency(&self) -> String {
        self.currency.clone()
    }
}
