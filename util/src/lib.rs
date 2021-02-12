#![warn(clippy::all)]
#![deny(clippy::correctness)]

pub mod iter;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod tree_vec;

pub fn compare_float(a: f64, b: f64) -> bool {
    (a - b).abs() <= f64::EPSILON
}
pub fn aggregate_error<T, U, E>(left: Result<T, E>, right: Result<U, E>) -> Result<(T, U), E>
where
    E: Extend<<E as IntoIterator>::Item> + IntoIterator,
{
    match (left, right) {
        (Ok(left), Ok(right)) => Ok((left, right)),
        (Err(mut left), Err(right)) => {
            left.extend(right);
            Err(left)
        }
        (Err(err), _) | (_, Err(err)) => Err(err),
    }
}
#[macro_export]
macro_rules! assert_iter {
    ($iter:expr, $($item:expr),* $(,)?) => {{
        use std::iter::Iterator;
        use std::option::Option;
        let mut iter = $iter;
        $(
            std::assert_eq!(Iterator::next(&mut iter), Option::Some($item));
        )*
        std::assert_eq!(Iterator::next(&mut iter), Option::None);
    }};
}
