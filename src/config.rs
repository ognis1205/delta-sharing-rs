mod fetcher;

use std::str::FromStr;

use once_cell::sync::Lazy;

use crate::server::utilities::bootstrap::HmacHasher;

pub(crate) static AWS_PROFILE: &str = "default";

pub(crate) static AWS_REGION: &str = "us-east-1";

pub(crate) static SECRET: Lazy<String> = Lazy::new(|| fetch::<String>("secret"));

pub(crate) static HASHER: Lazy<HmacHasher> = Lazy::new(|| {
    let hasher = fetch::<String>("hasher");
    HmacHasher::from_str(&hasher).unwrap_or(HmacHasher::Sha256)
});

pub fn fetch<T>(flag: &str) -> T
where
    fetcher::Flag<String>: fetcher::Fetch<T>,
{
    let config = fetcher::CONFIG.clone();
    let flag = fetcher::Flag {
        key: String::from(flag),
    };
    <fetcher::Flag<String> as fetcher::Fetch<T>>::fetch(&flag, &config)
}
