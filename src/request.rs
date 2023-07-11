use curl::easy::Easy;

use crate::MIN_YEAR_AVAIL;

pub fn fetch_csv_year(year: i32) -> Result<String, Box<dyn std::error::Error>> {
    let mut easy = Easy::new();
    let mut buffer = Vec::new();

    easy.url(treasury_url(year)?.as_str()).unwrap();

    let mut transfer = easy.transfer();
    transfer.write_function(|data| {
        buffer.extend_from_slice(data);
        Ok(data.len())
    })?;
    transfer.perform()?;
    drop(transfer);

    Ok(String::from_utf8(buffer)?)
}

fn treasury_url(year: i32) -> Result<String, Box<dyn std::error::Error>> {
    if year < MIN_YEAR_AVAIL { return Err("Treasury Curve not available for years less than 1990".into())}
    Ok(format!("https://home.treasury.gov/resource-center/data-chart-center/interest-rates/daily-treasury-rates.csv/{year}/all?type=daily_treasury_yield_curve&page&_format=csv"))
}

#[cfg(test)]
mod tests {
    use crate::current_year;
    use super::*;

    #[test]
    fn fetch_treasury_csv_data() {
        assert!(fetch_csv_year(current_year()).is_ok());
    }
}