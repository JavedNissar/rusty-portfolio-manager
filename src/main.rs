use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs;
use std::fmt;
use std::cmp::Ordering;

#[derive(Clone, Serialize, Deserialize)]
struct Data {
    stocks: Vec<Stock>,
    annual_expenses: f64,
    target_retirement_age: i32,
    current_age: i32,
    target_growth_rate: f64,
    usd_to_cad_exchange_rate: f64,
    expected_contribution: f64,
}

#[derive(Clone, Serialize, Deserialize)]
struct Stock {
    symbol: String,
    quote: f64,
    number_of_shares: i64,
    target_allocation: f64,
    is_usd: bool,
}

#[derive(PartialEq)]
struct CalculationResult{
    symbol: String,
    new_number_of_shares: i64,
    cost: f64,
}

impl PartialOrd for CalculationResult{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cost.partial_cmp(&other.cost)
    }
}

impl fmt::Display for CalculationResult{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Buy {} shares in {} for a cost of {}", self.new_number_of_shares, self.symbol, self.cost)
    }
}

fn calc_value_of_stock(stock: &Stock, usd_to_cad_exchange_rate: f64) -> f64{
    if stock.is_usd {
        stock.quote * (stock.number_of_shares as f64) * usd_to_cad_exchange_rate
    }else{
        stock.quote * (stock.number_of_shares as f64)
    }
}

fn calc_portfolio_val(stocks: Vec<Stock>, usd_to_cad_exchange_rate: f64) -> f64 {
    stocks.iter().map(|stock| calc_value_of_stock(stock, usd_to_cad_exchange_rate)).sum()
}

fn calc_number_of_shares_to_buy(stock: &Stock, total_value: f64, current_contribution_amount: f64) -> Option<CalculationResult>{
    let target_allocation_as_decimal = stock.target_allocation / 100.0;
    let new_number_of_shares = (target_allocation_as_decimal * total_value) / stock.quote;
    let shares_to_buy = new_number_of_shares - (stock.number_of_shares as f64);
    let cost = shares_to_buy * stock.quote;
    
    if shares_to_buy > 0.0 && cost < current_contribution_amount {
        Some(CalculationResult{
            symbol: stock.symbol.clone(),
            new_number_of_shares: (shares_to_buy as i64),
            cost: shares_to_buy * stock.quote,
        })
    } else if shares_to_buy > 0.0 {
        Some(determine_result_based_on_contrib_amount(stock, current_contribution_amount))
    }else {
        None
    }
}

fn determine_result_based_on_contrib_amount(stock: &Stock, current_contribution_amount: f64) -> CalculationResult{
   let shares_to_buy = current_contribution_amount / stock.quote; 
   CalculationResult{
       symbol: stock.symbol.clone(),
       new_number_of_shares: (shares_to_buy as i64),
       cost: shares_to_buy * stock.quote,
   }
}

fn print_where_to_contribute(data: Data){
    let mut current_contribution_amount = data.expected_contribution;
    let total_value = calc_portfolio_val(data.stocks.clone(), data.usd_to_cad_exchange_rate);
    let mut results: Vec<CalculationResult> = data.stocks.iter().filter_map(|stock| calc_number_of_shares_to_buy(stock, total_value,current_contribution_amount)).collect();
    results.sort_by(|a, b| a.partial_cmp(b).unwrap());

    println!("To fix allocations, make the following purchases");
    for result in results {
        if result.cost > current_contribution_amount { continue; }
        println!("{}", result);
        current_contribution_amount -= result.cost;
    }

    println!("Use extra contribution cash to do the following");
    for stock in data.stocks {
        if current_contribution_amount <= 0.0 { continue; }
        let result = determine_result_based_on_contrib_amount(&stock, current_contribution_amount);
        println!("{}", result);
        current_contribution_amount -= result.cost;
    }
}

fn print_how_close_to_retirement(data: Data){
    let total_value = calc_portfolio_val(data.stocks, data.usd_to_cad_exchange_rate);
    let new_value = total_value + data.expected_contribution;
    let years_to_pass = data.target_retirement_age - data.current_age;
    let multiplier = 1.0 + (data.target_growth_rate / 100.0);
    let retirement_rule = 4.0;
    let target_retirement_portfolio_value = data.annual_expenses / (retirement_rule / 100.0);
    let fully_grown_portfolio = new_value * multiplier.powi(years_to_pass);
    let percentage_of_retirement_value = fully_grown_portfolio / target_retirement_portfolio_value * 100.0;
    println!("Assuming a withdrawal rate of {}%, you need {} to retire", retirement_rule, target_retirement_portfolio_value);
    println!("You will have contributed {}, which in {} years will be {}", new_value, years_to_pass, fully_grown_portfolio);
    println!("This means once you are {}, you will be at {} % of your target", data.target_retirement_age, percentage_of_retirement_value); 
} 

fn print_current_portfolio_state(data: Data){
    let total_value = calc_portfolio_val(data.stocks.clone(), data.usd_to_cad_exchange_rate);

    println!("The portfolio state is the following:");
    for stock in data.stocks {
        let current_allocation = calc_value_of_stock(&stock, data.usd_to_cad_exchange_rate)/ total_value * 100.0;
        println!("You have {} shares in {} for an allocation of {}", stock.number_of_shares, stock.symbol, current_allocation);
    }
}

fn main() -> Result<()>{
    let data_file = fs::read_to_string("data.json").expect("Unable to read file!");
    let data: Data = serde_json::from_str(&data_file)?;

    print_current_portfolio_state(data.clone());
    println!("Figure out where to contribute:");
    print_where_to_contribute(data.clone());
    println!("Your distance to retirement:");
    print_how_close_to_retirement(data);

    Ok(())
}