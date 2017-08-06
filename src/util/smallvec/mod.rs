use smallvec;
use smallvec::SmallVec;
use std::fmt;

mod tests;

error_chain! {
    errors {
        DiscardQtyExceedsLen(discard_qty: usize, len: usize, fmtd_smallvec: String) {
            description(concat!(
                "an attempt was made to use the function `",
                module_path!(), "::", stringify!(discard_front),
                "` to discard more elements from a `SmallVec` than that container contained"
            ))
            display(
                "An attempt was made to use the function `{}::{}` to discard {} elements from a \
                 `SmallVec` of length {} (contents: {})",
                module_path!(), stringify!(discard_front), discard_qty, len, fmtd_smallvec
            )
        }
    }
}

/// Removes and discards the specified number of elements from the front of the given `SmallVec`.
///
/// `discard_front(small_vec, n)` is intended to be equivalent to `v.drain(..n)`, where `v` is a
/// standard `Vec`.
///
/// The order in which elements will be dropped is unspecified.
pub fn discard_front<A>(vec: &mut SmallVec<A>, discard_qty: usize) -> Result<()>
where
    A: smallvec::Array,
    SmallVec<A>: fmt::Debug,
{
    ensure!(
        discard_qty <= vec.len(),
        ErrorKind::DiscardQtyExceedsLen(discard_qty, vec.len(), format!("{:?}", vec))
    );

    // discard_qty=6 vec=[0 1 2 3 4 5 6 7 8 9]; len() - discard_qty = 4
    // i=0 swap(0, 6) -> [6 1 2 3 4 5 0 7 8 9]
    // i=1 swap(1, 7) -> [6 7 2 3 4 5 0 1 8 9]
    // i=2 swap(2, 8) -> [6 7 8 3 4 5 0 1 2 9]
    // i=3 swap(3, 9) -> [6 7 8 9 4 5 0 1 2 3]
    //      pop() x 6 -> [6 7 8 9            ]

    // discard_qty=1 vec=[0 0 1]; len() - discard_qty = 2
    // i=0 swap(0, 1) -> [0 0 1]
    // i=1 swap(1, 2) -> [0 1 0]
    //      pop() x 1 -> [0 1  ]

    for i in 0..(vec.len() - discard_qty) {
        vec.swap(i, i + discard_qty);
    }

    for _ in 0..discard_qty {
        vec.pop();
    }

    Ok(())
}
