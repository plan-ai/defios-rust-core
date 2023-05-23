use crate::constants::CONSTANT_OF_PROPORTIONALITY;

pub fn calculate_mint(token_supply: u64, lamports_amount: u64) -> [u64; 2] {
    [1, 1]
}

pub fn calculate_cost(token_supply: u64, number_of_tokens: u64) -> u64 {
    let new_token_supply = token_supply / 1000;
    let sum = (new_token_supply + number_of_tokens) as i128;
    let precise_token_supply = new_token_supply as i128;
    let precise_number_of_tokens = number_of_tokens as i128;
    let current_value: i128 =
        (CONSTANT_OF_PROPORTIONALITY * i128::pow(precise_token_supply, 3)) / 3000;
    let future_value: i128 = ((CONSTANT_OF_PROPORTIONALITY * i128::pow(sum, 3)) / 3000) as i128;
    let change_in_value = future_value - current_value;
    change_in_value as u64
}
