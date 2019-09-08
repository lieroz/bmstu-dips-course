use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Currency {
    Rouble,
    Dollar,
    Euro,
}

#[derive(Serialize, Deserialize)]
pub struct Wallet<'a> {
    owner: &'a str,
    balance: u64,
    currency: Currency,
}

impl<'a> Wallet<'a> {
    pub fn new(owner: &'a str, balance: u64, currency: Currency) -> Wallet<'a>
    {
        Wallet {
            owner,
            balance,
            currency
        }
    }
}
