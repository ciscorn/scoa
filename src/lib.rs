//! SCOC (Simpler Cloud-optimized Chunks)

pub mod delta;
pub mod sfcurve;

use std::io::{Cursor, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::{Compression, bufread::GzDecoder, write::GzEncoder};
use itertools::Itertools;

use crate::delta::{delta_decode, delta_encode};

struct LookupTable {
    /// Arbitrary MONOTONIC ids of chunks
    ///
    /// Encoded in storage as a LEB128 list of delta-encoded values.
    pub chunk_ids: Vec<u64>,

    /// End positions of each chunk
    ///
    /// Encoded in storage as a LEB128 list of delta-encoded values.
    pub end_positions: Vec<u64>,
}

#[derive(thiserror::Error, Debug)]
pub enum ScocError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid header: {0}")]
    InvalidHeader(String),
    #[error("Decoding error occurred")]
    DecodeError,
    #[error("Insufficient header bytes")]
    InsufficientHeader,
}

pub struct ScocReader<U>
where
    U: TryFrom<Vec<u8>, Error = ScocError>,
{
    /// Length of the header
    header_length: u32,
    /// Number of chunks
    num_chunks: u32,
    /// Lookup table for binary searching chunks
    lookup_table: LookupTable,
    /// Arbitrary user data
    user_data: U,
}

impl<U> ScocReader<U>
where
    U: TryFrom<Vec<u8>, Error = ScocError>,
{
    pub fn from_header_bytes(bytes: &[u8]) -> Result<Self, ScocError> {
        if &bytes[0..4] != b"SCOC" {
            return Err(ScocError::InvalidHeader("magic must be 'SCOC'".to_string()));
        }
        if bytes.len() < 17 {
            return Err(ScocError::InsufficientHeader);
        }
        let header_length = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        if bytes.len() < header_length as usize {
            return Err(ScocError::InsufficientHeader);
        }

        // TODO: Remove Cursor
        let mut cursor = Cursor::new(&bytes[8..]);

        let version = cursor.read_u8()?;
        if version != 1 {
            return Err(ScocError::InvalidHeader(format!(
                "Unsupported version: {}",
                version
            )));
        }
        let num_chunks = cursor.read_u32::<LittleEndian>()?;
        let lookup_table_compressed_size = cursor.read_u32::<LittleEndian>()?;
        let lookup_table_compressed = {
            let mut buf = vec![0; lookup_table_compressed_size as usize];
            cursor.read_exact(&mut buf)?;
            buf
        };
        let user_data = {
            let mut buf = vec![0; (header_length - lookup_table_compressed_size - 17) as usize];
            cursor.read_exact(&mut buf)?;
            buf
        };

        let lookup_table = {
            let mut gzreader = GzDecoder::new(Cursor::new(&lookup_table_compressed));
            let mut chunk_ids = Vec::new();
            let mut end_positions = Vec::new();
            for _ in 0..num_chunks {
                chunk_ids.push(
                    leb128::read::unsigned(&mut gzreader).map_err(|_| ScocError::DecodeError)?,
                );
            }
            for _ in 0..num_chunks {
                end_positions.push(
                    leb128::read::unsigned(&mut gzreader).map_err(|_| ScocError::DecodeError)?,
                );
            }
            LookupTable {
                chunk_ids: delta_decode(chunk_ids, 1).collect(),
                end_positions: delta_decode(end_positions, 1).collect(),
            }
        };

        Ok(ScocReader {
            header_length,
            num_chunks,
            lookup_table,
            user_data: user_data.try_into()?,
        })
    }

    pub fn num_chunks(&self) -> u32 {
        self.num_chunks
    }

    pub fn user_data(&self) -> &U {
        &self.user_data
    }

    pub fn header_length(&self) -> u32 {
        self.header_length
    }

    pub fn bisect_range(&self, id_begin: u64, id_end: u64) -> Option<Chunks> {
        let idx_begin = self
            .lookup_table
            .chunk_ids
            .partition_point(|&v| v < id_begin);
        let idx_end = self.lookup_table.chunk_ids.partition_point(|&v| v < id_end);
        match idx_begin != idx_end {
            true => Some(Chunks::new(
                &self.lookup_table,
                self.header_length,
                idx_begin as u32,
                idx_end as u32,
            )),
            false => None,
        }
    }
}

pub struct Chunks<'a> {
    lookup_table: &'a LookupTable,
    body_offset: u32,
    idx_begin: u32,
    idx_end: u32,
    pos_begin: u64,
    pos_end: u64,
}

impl<'a> Chunks<'a> {
    fn new(lookup_table: &'a LookupTable, body_offset: u32, idx_begin: u32, idx_end: u32) -> Self {
        let pos_begin = match idx_begin {
            0 => 0,
            _ => lookup_table.end_positions[(idx_begin - 1) as usize],
        };

        let pos_end = if idx_end == lookup_table.chunk_ids.len() as u32 {
            *lookup_table.end_positions.last().unwrap()
        } else {
            lookup_table.end_positions[idx_end as usize]
        };

        Chunks {
            lookup_table,
            body_offset,
            idx_begin,
            idx_end,
            pos_begin,
            pos_end,
        }
    }

    pub fn idx_begin(&self) -> u32 {
        self.idx_begin
    }
    pub fn idx_end(&self) -> u32 {
        self.idx_end
    }

    pub fn body_begin(&self) -> u64 {
        self.body_offset as u64 + self.pos_begin
    }
    pub fn body_end(&self) -> u64 {
        self.body_offset as u64 + self.pos_end
    }

    pub fn body_size(&self) -> usize {
        (self.pos_end - self.pos_begin) as usize
    }

    pub fn iter_chunks<'b>(&self, buf: &'b [u8]) -> impl Iterator<Item = &'b [u8]> {
        let mut prev_pos_end = self.pos_begin;
        self.lookup_table.chunk_ids[self.idx_begin as usize..self.idx_end as usize]
            .iter()
            .zip_eq(
                &self.lookup_table.end_positions[self.idx_begin as usize..self.idx_end as usize],
            )
            .map(move |(&_chunk_id, &end_position)| {
                let (start, end) = (prev_pos_end, end_position);
                prev_pos_end = end;
                &buf[(start - self.pos_begin) as usize..(end - self.pos_begin) as usize]
            })
    }
}

pub fn compress_lookup_table(
    chunk_ids: Vec<u32>,
    end_positions: Vec<u32>,
) -> std::io::Result<Vec<u8>> {
    let lookup_table_compressed = {
        let mut bin_tbl = vec![];
        for v in delta_encode(chunk_ids, 1) {
            leb128::write::unsigned(&mut bin_tbl, v as u64)?;
        }
        for v in delta_encode(end_positions, 1) {
            leb128::write::unsigned(&mut bin_tbl, v as u64)?;
        }
        let mut writer = GzEncoder::new(vec![], Compression::default());
        writer.write_all(&bin_tbl)?;
        writer.finish()?
    };
    Ok(lookup_table_compressed)
}

pub fn write_header(
    mut writer: impl WriteBytesExt,
    chunk_ids: Vec<u32>,
    end_positions: Vec<u32>,
    user_data: Vec<u8>,
) -> std::io::Result<()> {
    writer.write_all(b"SCOC")?;
    let num_chunks = chunk_ids.len() as u32;
    let lookup_table_compressed = compress_lookup_table(chunk_ids, end_positions)?;
    writer
        .write_u32::<LittleEndian>((17 + lookup_table_compressed.len() + user_data.len()) as u32)?;
    writer.write_u8(1)?;
    writer.write_u32::<LittleEndian>(num_chunks)?;
    writer.write_u32::<LittleEndian>(lookup_table_compressed.len() as u32)?;
    writer.write_all(&lookup_table_compressed)?;
    writer.write_all(&user_data)?;
    Ok(())
}
