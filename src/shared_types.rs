use std::error::Error;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Box<dyn Error + Send + Sync>>;
