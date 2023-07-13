//! US Treseary Yield Curve
//!
//! library fetches from https://home.treasury.gov/policy-issues/financing-the-government/interest-rate-statistics/legacy-interest-rate-xml-and-xsd-files
//!
//!
pub mod error;
pub mod request;
pub mod treasury_curve;
mod utility;

use error::TreasuryCurveError;
use request::fetch_csv_year;
use time::Date;
use treasury_curve::TreasuryCurveHistory;
use treasury_curve::{TreasuryCurve, TreasuryCurveCsv};
use utility::current_year;

const MIN_YEAR_AVAIL: i32 = 1990;

pub fn fetch_latest() -> Result<(Date, TreasuryCurve), TreasuryCurveError> {
    fetch_year(current_year())?.latest()
}

pub fn fetch_date(date: Date) -> Result<(Date, TreasuryCurve), TreasuryCurveError> {
    todo!()
}

pub fn fetch_year(year: i32) -> Result<TreasuryCurveHistory, TreasuryCurveError> {
    TreasuryCurveHistory::try_from(TreasuryCurveCsv(fetch_csv_year(year)?))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_latest_treasury_curve() {
        let latest = fetch_latest();
        assert!(latest.is_ok());
    }

    #[test]
    fn fetch_date_treasury_curve() {
        // data exists on this day
        let exist_date = Date::from_calendar_date(2023, time::Month::July, 5);
        //assert_eq!(fetch_date(exist_date).get("1 Mo"));
        // data does not exist on this day -> use the day prior
        let nonexist_date = Date::from_calendar_date(2023, time::Month::July, 2);
    }

    #[test]
    fn fetch_date_treasury_curve_date_does_not_exist() {
        todo!()
    }

    #[test]
    fn fetch_date_treasury_curve_data_point() {
        todo!()
    }
}
