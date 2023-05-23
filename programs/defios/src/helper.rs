pub fn verify_calc_buy(token_supply: u64, number_of_lamports: u64, token_amount: u64) -> bool {
    let new_token_amount = token_amount as u128;
    let new_token_supply = token_supply as u128;
    let value = u128::pow(new_token_amount, 2) + (2 * new_token_supply * new_token_amount);
    if (number_of_lamports as u128) == value {
        return true;
    };
    return false;
}

pub fn verify_calc_sell(token_supply: u64, number_of_lamports: u64, token_amount: u64) -> bool {
    let new_token_amount = token_amount as u128;
    let new_token_supply = token_supply as u128;
    let value = (2 * new_token_supply * new_token_amount) - u128::pow(new_token_amount, 2);
    if (number_of_lamports as u128) == value {
        return true;
    };
    return false;
}
