use rayon::prelude::*;
use wasm_bindgen::prelude::*;

pub use atlas_comms::set_panic_hook;
pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub fn crunch() -> u32 {
    (0..1_001).into_par_iter().map(|x| x * x).sum::<u32>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_worker);

    #[wasm_bindgen_test]
    fn sum_squares() {
        assert_eq!(crunch(), 333833500);
    }
}
