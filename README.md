# SCOC (Simpler Cloud-Optimized Chunks)

A minimal format for organizing arbitrary data chunks in a "cloud-optimized" way.

"Cloud-optimized" means that contiguous sub-chunks of interest can be retrieved with a single request (e.g., an HTTP Range request).

## Storage Format

- Header:
    - Lookup table for binary search over chunks
- Body is just a sequence of chunks:
    - Chunk
    - Chunk
    - Chunk
    - ...

```rust
// pseudo-code

struct _Header {
    /// "SCOC"
    magic: [u8; 4],

    /// Length of the header
    header_length: u32,

    /// Version (1)
    version: u8,

    /// Number of chunks
    num_chunks: u32,

    /// Size of the compressed lookup table
    lookup_table_compressed_size: u32,

    /// Lookup table (gzipped) for binary search over chunks
    lookup_table_compressed: Vec<u8>,

    /// Arbitrary user data
    user_data: Vec<u8>,

    // end of the header
}

struct _LookupTable {
    /// Arbitrary MONOTONIC ids of chunks
    ///
    /// Encoded in storage as a VarInt (LEB128) list of delta-encoded values.
    pub chunk_ids: Vec<u64>,

    /// End positions of each chunk
    ///
    /// Encoded in storage as a VarInt (LEB128) list of delta-encoded values.
    pub end_positions: Vec<u64>,
}
```
