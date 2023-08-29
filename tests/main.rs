use near_sdk::{env, AccountId};
use chrono::{NaiveDate, Duration, Datelike};

use fungible_token::Contract;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use fungible_token::investment::investment::{InvestmentInput, get_current_date, is_valid_date_format, add_month};
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::json_types::U128;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_234_567_891_011_121;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    #[should_panic(expected = "Only the contract owner can add an investor.")]
    fn test_add_investors_no_permission() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(2).into(), TOTAL_SUPPLY.into(), accounts(5).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .predecessor_account_id(accounts(1))
            .block_timestamp(1000)
            .build());

        let investment = InvestmentInput {
            start_date: "2023-01-01".to_string(),
            vesting: 10,
            total_amount: U128(TOTAL_SUPPLY),
            cycle: 1
        };
        
        contract.add_investor(accounts(1), investment.clone());
    }

    #[test]
    #[should_panic(expected = "Total amount cannot exceed total supply.")]
    fn test_add_investors_oversupply() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(2).into(), TOTAL_SUPPLY.into(), accounts(5).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .predecessor_account_id(accounts(2))
            .block_timestamp(1000)
            .build());

        let investment = InvestmentInput {
            start_date: "2023-01-01".to_string(),
            vesting: 10,
            total_amount: U128(TOTAL_SUPPLY + 1),
            cycle: 1
        };
        
        contract.add_investor(accounts(1), investment.clone());
    }

    #[test]
    #[should_panic(expected = "Invalid date format. Please use YYYY-MM-DD format.")]
    fn test_add_investors_date_regex() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(2).into(), TOTAL_SUPPLY.into(), accounts(5).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .predecessor_account_id(accounts(2))
            .block_timestamp(1000)
            .build());

        let investment = InvestmentInput {
            start_date: "2023-02-30".to_string(),
            vesting: 10,
            total_amount: U128(TOTAL_SUPPLY + 1),
            cycle: 1
        };
        
        contract.add_investor(accounts(1), investment.clone());
    }

    #[test]
    #[should_panic(expected = "Cycle must be a positive integer.")]
    fn test_add_investors_cycle() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(2).into(), TOTAL_SUPPLY.into(), accounts(5).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .predecessor_account_id(accounts(2))
            .block_timestamp(1000)
            .build());

        let investment = InvestmentInput {
            start_date: "2023-01-01".to_string(),
            vesting: 10,
            total_amount: U128(TOTAL_SUPPLY + 1),
            cycle: 0
        };
        
        contract.add_investor(accounts(1), investment.clone());
    }

    #[test]
    fn test_get_current_date() {
        let start_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2100, 12, 31).unwrap();

        let mut current_date = start_date;
        while current_date <= end_date {
            let timestamp = current_date.and_hms_opt(0, 0, 0).unwrap();
            testing_env!(get_context(accounts(0)).block_timestamp(timestamp.timestamp_nanos() as u64).build());
            let (year, month, day) = get_current_date();

            assert_eq!((year, month, day), (current_date.year(), current_date.month(), current_date.day()));

            current_date += Duration::days(1);
        }
    }

    #[test]
    fn test_add_month() {
        let day = 1;
        for year in 2000..2101 {
            for month in 1..13 {
                let (n_year, n_month, n_day) = add_month(year, month, day, 1);
                assert!(is_valid_date_format(&format!("{:04}-{:02}-{:02}", n_year, n_month, n_day)));
            }
        }

        let day = 1;
        for year in 2000..2101 {
            for month in 1..13 {
                let (n_year, n_month, n_day) = add_month(year, month, day, 3);
                assert!(is_valid_date_format(&format!("{:04}-{:02}-{:02}", n_year, n_month, n_day)));
            }
        }

        let day = 1;
        for year in 2000..2101 {
            for month in 1..13 {
                let (n_year, n_month, n_day) = add_month(year, month, day, 12);
                assert!(is_valid_date_format(&format!("{:04}-{:02}-{:02}", n_year, n_month, n_day)));
            }
        }
    }

    #[test]
    fn test_get_current_date_today() {
        testing_env!(get_context(accounts(0)).block_timestamp(1693274924000000000).build());
        let (year, month, day) = get_current_date();
        env::log_str(&format!("{:04}-{:02}-{:02}", year, month, day));
    }
}