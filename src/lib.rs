use reed_solomon::Error;
use reed_solomon::ReedSolomonDecoder;
use reed_solomon::ReedSolomonEncoder;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys;

#[wasm_bindgen]
pub struct Shard {
    index: u16,
    data: js_sys::Uint8Array,
}

#[wasm_bindgen]
impl Shard {
    #[wasm_bindgen(constructor)]
    pub fn new(index: u16, data: js_sys::Uint8Array) -> Shard {
        Shard { index, data }
    }

    #[wasm_bindgen(getter)]
    pub fn index(&self) -> u16 {
        self.index
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> js_sys::Uint8Array {
        self.data.clone()
    }
}

type RawShard = Vec<u8>;

fn rs_encode(
    original_count: usize,
    recovery_count: usize,
    shard_bytes: usize,
    shards: &[js_sys::Uint8Array],
) -> Result<Vec<RawShard>, Error> {
    let mut encoder = ReedSolomonEncoder::new(original_count, recovery_count, shard_bytes)?;

    for shard in shards {
        encoder.add_original_shard(shard.to_vec())?;
    }

    let result = encoder.encode()?;

    Ok(result
        .recovery_iter()
        .map(|v| v.to_vec())
        .collect())
}

fn rs_decode(
    original_count: usize,
    recovery_count: usize,
    shard_bytes: usize,
    // TODO [ToDr] this might be better of as one single vector,
    // because now we need to copy a lot from JS to Rust
    shards: &[Shard],
) -> Result<Vec<Shard>, Error> {
    let mut decoder = ReedSolomonDecoder::new(original_count, recovery_count, shard_bytes)?;

    for shard in shards {
        let idx_usize = shard.index as usize;
        if idx_usize < original_count {
            decoder.add_original_shard(idx_usize, shard.data.to_vec())?;
        } else {
            decoder.add_recovery_shard(idx_usize - original_count, shard.data.to_vec())?;
        }
    }

    let decoding_result = decoder.decode()?;

    Ok(decoding_result
        .restored_original_iter()
        .map(|(idx, slice)| Shard::new(idx as u16, slice.into()))
        .collect()
    )
}

#[wasm_bindgen]
pub fn encode(
    original_count: u16,
    recovery_count: u16,
    shard_bytes: u16,
    shards: Vec<js_sys::Uint8Array>,
) -> Result<Vec<Shard>, String> {
    let shards = rs_encode(
        original_count as usize, 
        recovery_count as usize, 
        shard_bytes as usize, 
        &shards
    ).map_err(|e| e.to_string())?;

    Ok(shards.into_iter().enumerate().map(|(idx, v)| Shard::new(idx as u16, v.as_slice().into())).collect())
}

#[wasm_bindgen]
pub fn decode(
    original_count: u16,
    recovery_count: u16,
    shard_bytes: u16,
    shards: Vec<Shard>,
) -> Result<Vec<Shard>, String> {
    let result = rs_decode(
        original_count as usize,
        recovery_count as usize,
        shard_bytes as usize,
        &shards
    );

    result.map_err(|e| e.to_string())
}
