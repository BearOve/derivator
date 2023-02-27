pub use derivator_derive::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayJoin<'a, T, Delim>(pub &'a [T], pub Delim);

impl<'a, T, Delim> fmt::Display for DisplayJoin<'a, T, Delim>
where
    T: fmt::Display,
    Delim: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self(slice, join) = self;
        for (i, arg) in slice.iter().enumerate() {
            if i > 0 {
                write!(f, "{join}{arg}")?;
            } else {
                write!(f, "{arg}")?;
            }
        }
        Ok(())
    }
}
