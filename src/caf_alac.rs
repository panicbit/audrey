use super::read::FormatError;
use alac::{Decoder, StreamInfo};
use caf::{chunks::CafChunk, CafPacketReader, ChunkType, FormatType};
use std::io::{Read, Seek};

pub struct AlacReader<T>
where
    T: Read + Seek,
{
    pub caf_reader: CafPacketReader<T>,
    pub alac_decoder: Decoder,
}

impl<T> AlacReader<T>
where
    T: Read + Seek,
{
    /// Creates a new AlacReader
    ///
    /// Returns Err(..) on IO errors, or if the stream is not CAF.
    /// Returns Ok(Some(..)) if the format inside is ALAC,
    /// None if its not ALAC.
    pub fn new(rdr: T) -> Result<Option<Self>, FormatError> {
        let caf_reader = CafPacketReader::new(rdr, vec![ChunkType::MagicCookie])?;
        if caf_reader.audio_desc.format_id != FormatType::AppleLossless {
            return Ok(None);
        }
        let cookie = caf_reader
            .chunks
            .iter()
            .filter_map(|c| match c {
                CafChunk::MagicCookie(ref d) => Some(d.clone()),
                _ => None,
            })
            .next()
            .unwrap();
        let stream_info = StreamInfo::from_cookie(&cookie).map_err(|_| FormatError::Alac(()))?;
        Ok(Some(AlacReader {
            caf_reader,
            alac_decoder: Decoder::new(stream_info),
        }))
    }
    pub fn read_packet(&mut self) -> Result<Option<Vec<i32>>, FormatError> {
        let mut output_buf: Vec<i32> = vec![
            0;
            (self.caf_reader.audio_desc.frames_per_packet
                * self.caf_reader.audio_desc.channels_per_frame)
                as usize
        ];
        let packet = match self.caf_reader.next_packet()? {
            Some(pck) => pck,
            None => return Ok(None),
        };
        self.alac_decoder
            .decode_packet(&packet, &mut output_buf)
            .map_err(|_| FormatError::Alac(()))?;
        Ok(Some(output_buf))
    }
}
