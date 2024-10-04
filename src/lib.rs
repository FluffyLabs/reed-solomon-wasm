use reed_solomon::Error;
use reed_solomon::ReedSolomonDecoder;
use reed_solomon::ReedSolomonEncoder;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys;

#[wasm_bindgen]
pub struct ShardsCollection {
    length: u32,
    shard_len: u16,
    data: js_sys::Uint8Array,
    indices: Option<js_sys::Uint16Array>,
}

#[wasm_bindgen]
impl ShardsCollection {
    #[wasm_bindgen(constructor)]
    pub fn new(shard_len: u16, data: js_sys::Uint8Array, indices: Option<js_sys::Uint16Array>) -> Self {
        let length = data.length() / shard_len as u32;
        if let Some(ref indices) = indices {
            if indices.length() != length {
                panic!("Mismatching indices and data length.");
            }
        }

        Self { length, shard_len, indices, data }
    }

    #[wasm_bindgen(getter)]
    pub fn len(&self) -> usize {
        self.length as usize
    }

    #[wasm_bindgen(getter)]
    pub fn chunk_at(&self, index: usize) -> js_sys::Uint8Array {
        let begin = index as u32 * self.shard_len as u32;
        let end = begin + self.shard_len as u32;
        self.data.subarray(begin, end)
    }

    #[wasm_bindgen(getter)]
    pub fn chunk_index_at(&self, index: usize) -> u16 {
        self.indices
            .as_ref()
            .map(|v| v.at(index as i32).expect("Out of bounds access to indices."))
            .unwrap_or(index as u16)
    }
}


struct RsShardsCollection {
    pub length: usize,
    pub shard_len: u16,
    data: Vec<u8>,
    indices: Option<Vec<u16>>,
}

impl RsShardsCollection {
    pub fn chunk_at(&self, index: usize) -> &[u8] {
        let begin = index * self.shard_len as usize;
        let end = begin + self.shard_len as usize;
        
        &self.data[begin..end]
    }

    pub fn chunk_index_at(&self, index: usize) -> u16 {
        self.indices.as_ref().map(|v| v[index]).unwrap_or(index as u16)
    }
}

impl From<RsShardsCollection> for ShardsCollection {
    fn from(value: RsShardsCollection) -> Self {
        let RsShardsCollection { length, shard_len, data, indices } = value;

        Self {
            length: length as u32,
            shard_len,
            data: data.as_slice().into(),
            indices: indices.map(|i| i.as_slice().into()),
        }
    }
}

impl From<ShardsCollection> for RsShardsCollection {
    fn from(value: ShardsCollection) -> Self {
        let ShardsCollection { length, shard_len, data, indices } = value;

        Self {
            length: length as usize,
            shard_len,
            data: data.to_vec(),
            indices: indices.map(|v| v.to_vec()),
        }
    }
}

fn rs_encode(
    recovery_count: usize,
    shard_bytes: usize, // TODO [ToDr] is this the same as `shards.shard_len`?
    shards: RsShardsCollection,
) -> Result<RsShardsCollection, Error> {
    let mut encoder = ReedSolomonEncoder::new(shards.length, recovery_count, shard_bytes)?;

    for i in 0..shards.length {
        encoder.add_original_shard(shards.chunk_at(i))?;
    }

    let result = encoder.encode()?;

    let mut data = Vec::with_capacity(recovery_count * shards.shard_len as usize);

    for chunk in result.recovery_iter() {
        data.extend(chunk);
    }

    Ok(RsShardsCollection{
        length: recovery_count,
        shard_len: shards.shard_len,
        data,
        indices: None,
    })
}

fn rs_decode(
    original_count: usize,
    recovery_count: usize,
    shard_bytes: usize,
    shards: RsShardsCollection,
) -> Result<RsShardsCollection, Error> {
    let mut decoder = ReedSolomonDecoder::new(original_count, recovery_count, shard_bytes)?;

    for i in 0..shards.length {
        let idx = shards.chunk_index_at(i) as usize;
        let data = shards.chunk_at(i);
        if idx < original_count {
            decoder.add_original_shard(idx, data)?;
        } else {
            decoder.add_recovery_shard(idx - original_count, data)?;
        }
    }

    let decoding_result = decoder.decode()?;

    let mut indices = vec![];
    let mut data = vec![];
    for (idx, shard) in decoding_result.restored_original_iter() {
        indices.push(idx as u16);
        data.extend(shard);
    }

    Ok(RsShardsCollection {
        length: data.len(),
        shard_len: shards.shard_len,
        indices: Some(indices),
        data,
    })
}

#[wasm_bindgen]
pub fn encode(
    recovery_count: u16,
    shard_bytes: u16,
    shards: ShardsCollection,
) -> Result<ShardsCollection, String> {
    let result = rs_encode(
        recovery_count as usize, 
        shard_bytes as usize, 
        shards.into()
    ).map_err(|e| e.to_string())?;

    Ok(result.into())
}

#[wasm_bindgen]
pub fn decode(
    original_count: u16,
    recovery_count: u16,
    shard_bytes: u16,
    shards: ShardsCollection,
) -> Result<ShardsCollection, String> {
    let result = rs_decode(
        original_count as usize,
        recovery_count as usize,
        shard_bytes as usize,
        shards.into(),
    ).map_err(|e| e.to_string())?;

    Ok(result.into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_encode_shards() {
        // TODO [ToDr]
    }
}
