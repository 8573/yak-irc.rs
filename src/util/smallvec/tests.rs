#![cfg(test)]

use super::*;
use quickcheck::TestResult;
use std;

#[test]
fn discard_front_1() {
    let mut vec = SmallVec::<[u32; 8]>::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9][..]);
    discard_front(&mut vec, 6).unwrap();
    assert_eq!(vec[..], [6, 7, 8, 9]);
}

#[test]
fn discard_front_2() {
    let mut vec = SmallVec::<[u32; 8]>::from(&[0, 0, 1][..]);
    discard_front(&mut vec, 1).unwrap();
    assert_eq!(vec[..], [0, 1]);
}

quickcheck! {
    fn discard_front_matches_vec_drain_1(input_vec: Vec<u32>, discard_qty: usize) -> TestResult {
        if discard_qty > input_vec.len() {
            return TestResult::discard();
        }

        let mut small_vec = SmallVec::<[u32; 8]>::from_vec(input_vec.clone());
        let mut std_vec = input_vec;

        discard_front(&mut small_vec, discard_qty).unwrap();
        let std::vec::Drain { .. } = std_vec.drain(..discard_qty);

        assert_eq!(small_vec[..], std_vec[..]);
        TestResult::passed()
    }
}
