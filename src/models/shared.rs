use fraction::Decimal;

// Aiming to use the type with the most precision available
// and consider rounding as a presentation concern.
pub type Amount = Decimal;

// Let's say u64 are okay values for hypergrowth at this time :)
pub type OID = u64;
