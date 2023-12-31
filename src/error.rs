use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TreasuryCurveError {
    #[error("data label not recognized: {0}")]
    MissingLabel(String),
    #[error("no data before the year 1990 or greater than current year, using: {0}")]
    InvalidYear(i32),
    #[error("requested date is outside the range of data: {0}")]
    OutsideDateRange(String),
    #[error("fetch error - could NOT access and get data from web")]
    FetchData(#[from] curl::Error),
    #[error("trouble parsing data from web into utf8")]
    WebParseUtf8(#[from] std::string::FromUtf8Error),
}
