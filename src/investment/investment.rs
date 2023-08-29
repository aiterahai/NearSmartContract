use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId, Balance, env, Gas, Promise, PromiseError, log};
use near_sdk::json_types::U128;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::*;

pub const NO_DEPOSIT: u128 = 1;
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(30_000_000_000_000);

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
pub struct Investment {
    pub start_date: String,
    pub vesting: u8,
    pub cycle: u8,
    pub remaining_payouts: u8,
    pub total_amount: Balance,
    pub paid_amount: Balance,
    pub last_payment_date: String
}

#[derive(Serialize, Deserialize)]
pub struct InvestmentJson {
    pub start_date: String,
    pub vesting: u8,
    pub cycle: u8,
    pub remaining_payouts: u8,
    pub total_amount: String,
    pub paid_amount: String,
    pub last_payment_date: String
}

impl From<Investment> for InvestmentJson {
    fn from(investment: Investment) -> Self {
        let total_amount = investment.total_amount.to_string();
        let paid_amount = investment.paid_amount.to_string();

        InvestmentJson {
            start_date: investment.start_date,
            vesting: investment.vesting,
            cycle: investment.cycle,
            remaining_payouts: investment.remaining_payouts,
            total_amount,
            paid_amount,
            last_payment_date: investment.last_payment_date
        }
    }
}

impl From<InvestmentJson> for Investment {
    fn from(investment_json: InvestmentJson) -> Self {
        let total_amount = investment_json.total_amount.parse::<u128>().expect("Failed to parse total_amount from String to u128");
        let paid_amount = investment_json.paid_amount.parse::<u128>().expect("Failed to parse paid_amount from String to u128");

        Investment {
            start_date: investment_json.start_date,
            vesting: investment_json.vesting,
            cycle: investment_json.cycle,
            remaining_payouts: investment_json.remaining_payouts,
            total_amount,
            paid_amount,
            last_payment_date: investment_json.last_payment_date
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
pub struct InvestmentInput {
    pub start_date: String,
    pub vesting: u8,
    pub cycle: u8,
    pub total_amount: U128,
}

#[near_bindgen]
impl Contract {
    pub fn get_all_investors(&self) -> Vec<(AccountId, Investment)> {
        let mut result: Vec<(AccountId, Investment)> = Vec::new();
        for (account_id, investment) in self.investors.iter() {
            result.push((account_id, investment));
        }
        result
    }

    pub fn add_investor(&mut self, account_id: AccountId, investment: InvestmentInput) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only the contract owner can add an investor.");
        
        assert!(is_valid_date_format(&investment.start_date), "Invalid date format. Please use YYYY-MM-DD format.");

        assert!(investment.cycle > 0, "Cycle must be a positive integer.");

        assert!(investment.vesting > 0, "Vesting must be a positive integer.");

        let converted_investment = Investment {
            start_date: investment.start_date.clone(),
            vesting: investment.vesting,
            cycle: investment.cycle,
            remaining_payouts: investment.vesting / investment.cycle,
            total_amount: investment.total_amount.into(),
            paid_amount: 0,
            last_payment_date: investment.start_date
        };

        assert!(converted_investment.total_amount > 0, "Total amount must be a positive integer.");

        assert!(converted_investment.total_amount <= self.total_supply, "Total amount cannot exceed total supply.");

        self.investors.insert(&account_id, &converted_investment);
    }

    #[payable]
    pub fn distribute_token(&mut self) {
        let investor_keys: Vec<AccountId> = self.investors.keys().collect();
        for investor_account_id in investor_keys {
            if let Some(mut investment) = self.investors.get(&investor_account_id) {
                if investment.remaining_payouts == 0 {
                    continue;
                }

                let (current_year, currnet_month, current_day) = get_current_date();

                let (next_year, next_month, next_day) = if investment.vesting / investment.cycle == investment.remaining_payouts {
                    let components: Vec<&str> = investment.start_date.split('-').collect();
                    (components[0].parse::<i32>().unwrap(), components[1].parse::<u32>().unwrap(), components[2].parse::<u32>().unwrap())
                } else {
                    let components: Vec<&str> = investment.last_payment_date.split('-').collect();
                    add_month(components[0].parse::<i32>().unwrap(),components[1].parse::<u32>().unwrap(),components[2].parse::<u32>().unwrap(), investment.cycle)
                };

                if (next_year as u32) * 10000 + next_month * 100 + next_day > (current_year as u32) * 10000 + currnet_month * 100 + current_day {
                    continue;
                }

                let amount = if investment.remaining_payouts == 1 {
                    investment.total_amount - investment.paid_amount
                } else {
                    (investment.total_amount / investment.vesting as u128) * investment.cycle as u128
                };

                if investment.vesting != investment.remaining_payouts {
                    investment.last_payment_date = format!("{:04}-{:02}-{:02}", next_year, next_month, next_day);
                }
                investment.remaining_payouts -= 1;
                investment.paid_amount += amount;

                Promise::new(self.token_contract_address.clone())
                .function_call(
                    "ft_transfer".to_string(),
                    json!({
                        "receiver_id": investor_account_id,
                        "amount": amount.to_string(),
                        "memo": Option::<String>::None
                    })
                    .to_string()
                    .as_bytes()
                    .to_vec(),
                    NO_DEPOSIT,
                    GAS_FOR_FT_TRANSFER,
                ).then(Self::ext(env::current_account_id()).on_ft_transfer_success(investor_account_id, InvestmentJson::from(investment)));
            }
        }
    }


    #[private]
    pub fn on_ft_transfer_success(&mut self, #[callback_result] last_result: Result<(), PromiseError>, investor_account_id: AccountId, investment: InvestmentJson) {
        let investment = Investment::from(investment);
        match last_result {
            Ok(_) => {
                self.investors.insert(&investor_account_id, &investment);
                log!("on_ft_transfer_success: FT transfer has been successful result");
            }
            Err(_) => {
                log!("on_ft_transfer_success: FT transfer has failed with error");
            }
        }
    }
}

pub fn add_month(year: i32, month: u32, day: u32, cycle: u8) -> (i32, u32, u32) {
    let mut next_year = year;
    let mut next_month = month + cycle as u32 - 1;
    let next_day = day;

    next_year += next_month as i32 / 12;
    next_month = next_month % 12 + 1;

    (next_year, next_month, next_day)
}

pub fn get_current_date() -> (i32, u32, u32) {
    let timestamp = env::block_timestamp() / 1000000000;

    let seconds_in_day = 24 * 60 * 60;

    let mut days_since_epoch = timestamp / seconds_in_day;

    let mut years = 1970;
    while days_since_epoch >= 365 {
        if is_leap_year(years) {
            if days_since_epoch >= 366 {
                days_since_epoch -= 366;
                years += 1;
            } else {
                break;
            }
        } else {
            days_since_epoch -= 365;
            years += 1;
        }
    }

    let mut months_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if is_leap_year(years) {
        months_days[2] = 29;
    }

    let mut months = 1;
    for (i, days) in months_days.iter().enumerate().skip(1) {
        if days_since_epoch < *days {
            months = i as u32;
            break;
        } else {
            days_since_epoch -= *days;
        }
    }

    let days = days_since_epoch + 1;

    (years as i32,months as u32, days as u32)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn is_valid_date_format(date: &str) -> bool {
    if date.len() != 10 {
        return false;
    }

    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return false;
    }

    let year = parts[0].parse::<i32>();
    let month = parts[1].parse::<i32>();
    let day = parts[2].parse::<i32>();

    if year.is_err() || month.is_err() || day.is_err() {
        return false;
    }

    let year = year.unwrap();
    let month = month.unwrap();
    let day = day.unwrap();


    if year < 2000 || year > 9999 || month < 1 || month > 12 || day < 1 {
        return false;
    }
    
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => day <= 31,
        4 | 6 | 9 | 11 => day <= 30,
        2 => {
            if is_leap_year(year) {
                day <= 29
            } else {
                day <= 28
            }
        },
        _ => false,
    }
}
