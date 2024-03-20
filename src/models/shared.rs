use fraction::Decimal;

// Aiming to use the type with the most precision available
// and consider rounding as a presentation concern.
pub type Amount = Decimal;

pub type TransactionID = u64;

// As specified by Rust Test.pdf
pub type ClientID = u16;