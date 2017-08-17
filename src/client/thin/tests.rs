#![cfg(test)]

use super::*;
use quickcheck::TestResult;

quickcheck! {
    fn event_context_id_mio_token_conversion_bijective_1(n1: usize, n2: usize) -> TestResult {
        let mio_tok_1 = mio::Token(n1);
        let mio_tok_2 = mio::Token(n2);

        if mio_tok_1 == mio_tok_2 {
            return TestResult::discard();
        }

        let evt_ctx_id_1 = EventContextId::from(mio_tok_1);
        let evt_ctx_id_2 = EventContextId::from(mio_tok_2);

        TestResult::from_bool(evt_ctx_id_1 != evt_ctx_id_2)
    }

    fn event_context_id_mio_token_conversion_roundtrip_1(token_number: usize) -> TestResult {
        let mio_tok_1 = mio::Token(token_number);
        let evt_ctx_id = EventContextId::from(mio_tok_1);
        let mio_tok_2 = match evt_ctx_id.as_mio_token() {
            Ok(t) => t,
            Err(_) => return TestResult::discard(),
        };

        TestResult::from_bool(mio_tok_1 == mio_tok_2)
    }

    fn event_context_id_mio_token_conversion_roundtrip_2(token_number: usize) -> TestResult {
        let mio_tok_1 = mio::Token(token_number);
        let evt_ctx_id_1 = EventContextId::from(mio_tok_1);
        let mio_tok_2 = match evt_ctx_id_1.as_mio_token() {
            Ok(t) => t,
            Err(_) => return TestResult::discard(),
        };
        let evt_ctx_id_2 = EventContextId::from(mio_tok_2);

        TestResult::from_bool(evt_ctx_id_1 == evt_ctx_id_2)
    }
}
