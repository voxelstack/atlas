use rayon::prelude::*;
use wasm_bindgen::prelude::*;

pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub fn crunch() -> u32 {
    console_error_panic_hook::set_once();
    (0..1_001).into_par_iter().map(|x| x * x).sum::<u32>()
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;

    use super::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_worker);

    #[wasm_bindgen_test]
    fn pass() {
        assert_eq!(crunch(), 333833500);
    }
}
