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

pub fn fetch_date(request_date: Date) -> Result<(Date, TreasuryCurve), TreasuryCurveError> {
    fetch_year(request_date.year())?.from_date(request_date)
}

pub fn fetch_year(requst_year: i32) -> Result<TreasuryCurveHistory, TreasuryCurveError> {
    TreasuryCurveHistory::try_from(TreasuryCurveCsv(fetch_csv_year(requst_year)?))
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
        let exist_date = Date::from_calendar_date(2023, time::Month::July, 5).unwrap();
        assert_eq!(fetch_date(exist_date).unwrap().0, exist_date);
        // data does not exist on this day Jul 2 is a weeekend -> use the day prior which is June 30
        let nonexist_date = Date::from_calendar_date(2023, time::Month::July, 2).unwrap();
        let nonexist_date_check = Date::from_calendar_date(2023, time::Month::June, 30).unwrap();
        assert_eq!(fetch_date(nonexist_date).unwrap().0, nonexist_date_check);
    }

    #[test]
    fn fetch_date_treasury_curve_date_does_not_exist() {
        // make sure I'm getting an error for a bad year
        todo!()
    }

    #[test]
    fn fetch_date_treasury_curve_for_various_test_dates() {
        // pick 6 dates in history and check each date against 2 points in the curve
        // (1)use old dates, (2) weekend, (3) holiday, (4) recent date, (5) end of year, (6) beginning of year
        todo!()
    }
}
