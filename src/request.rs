use curl::easy::Easy;
use crate::{MIN_YEAR_AVAIL, error::TreasuryCurveError, current_year};

pub fn fetch_csv_year(year: i32) -> Result<String, TreasuryCurveError> {
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

fn treasury_url(year: i32) -> Result<String, TreasuryCurveError> {
    if (year < MIN_YEAR_AVAIL) || (year > current_year()) { return Err(TreasuryCurveError::InvalidYear(year))}
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