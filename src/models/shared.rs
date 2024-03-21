use fraction::Decimal;

// Aiming to use the type with the most precision available
// and considering rounding as a presentation concern.
pub type Amount = Decimal;

// As specified by `Rust Test.pdf``
pub type TransactionID = u32;

// As specified by `Rust Test.pdf``
pub type ClientID = u16;
