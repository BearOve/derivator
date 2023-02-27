use crate::*;

#[macro_export]
macro_rules! error {
    ($span:expr, $msg:literal $($args:tt)*) => {{
        Error::new($span.span(), format_args!($msg $($args)*))
    }}
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
