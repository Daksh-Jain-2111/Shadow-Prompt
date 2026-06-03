use std::time::Instant;

#[derive(Clone, Debug)]
pub struct PerfSample {
    pub mask_ms: u128,
    pub input_len: usize,
    pub output_len: usize,
}

pub fn mask_text_profiled(input: String) -> (String, PerfSample) {
    let input_len = input.len();
    let t0 = Instant::now();
    let out = crate::mask_text(input);
    let mask_ms = t0.elapsed().as_millis();
    let output_len = out.len();
    (
        out,
        PerfSample {
            mask_ms,
            input_len,
            output_len,
        },
    )
}
