use anchor_lang::prelude::Pubkey;
use anchor_spl::metadata::mpl_token_metadata::ID;
//helper functions
#[inline(always)]
pub fn calculate_buy_amount(token_supply: u64, token_amount: u64) -> u128 {
    let new_token_amount = token_amount as u128;
    let new_token_supply = token_supply as u128;
    let value = u128::pow(new_token_amount, 2) + (2 * new_token_supply * new_token_amount);
    value
}

#[inline(always)]
pub fn check_equal_buy(lamports: u128, ideal_lamports: u128) -> bool {
    if lamports >= ideal_lamports && lamports - ideal_lamports <= 2 {
        return true;
    };
    return false;
}

#[inline(always)]
pub fn calculate_sell_amount(token_supply: u64, token_amount: u64) -> u128 {
    let new_token_amount = token_amount as u128;
    let new_token_supply = token_supply as u128;
    let first_value = 2 * new_token_supply * new_token_amount;
    let second_value = u128::pow(new_token_amount, 2);
    if second_value > first_value {
        return 0;
    } else {
        return first_value - second_value;
    }
}

//main check code functions
pub fn verify_calc_buy(token_supply: u64, number_of_lamports: u64, token_amount: u64) -> bool {
    let value = calculate_buy_amount(token_supply, token_amount);
    check_equal_buy(number_of_lamports as u128, value)
}

pub fn verify_calc_sell(token_supply: u64, number_of_lamports: u64, token_amount: u64) -> bool {
    let value = calculate_sell_amount(token_supply, token_amount);
    if (number_of_lamports as u128) == value {
        return true;
    };
    return false;
}

pub fn verify_swap(
    token_supply1: u64,
    token_supply2: u64,
    token_amount1: u64,
    token_amount2: u64,
) -> bool {
    let value1 = calculate_sell_amount(token_supply1, token_amount1);
    let value2 = calculate_buy_amount(token_supply2, token_amount2);
    if value1 == value2 {
        return true;
    }
    false
}

pub const PREFIX: &str = "metadata";

pub fn find_metadata_account(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PREFIX.as_bytes(), ID.as_ref(), mint.as_ref()], &ID)
}
