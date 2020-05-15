use std::convert::TryFrom;
pub mod err;
pub use err::RErr;

/// Sampleformat to code audio data with. Variants
/// starting with `Int` mean "signed integer" or `i`
/// (prefix in Rust) and `Float` mean "floating point"
/// or `f`.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Fmt {
  Int8    = 0, Int16   = 1,
  Int32   = 2, Int64   = 3,
  Float32 = 4, Float64 = 5
}

impl TryFrom<u8> for Fmt {
  type Error = RErr;
  fn try_from(f: u8) -> Result<Self, Self::Error> {
    match f {
      0 => Ok(Fmt::Int8), 1 => Ok(Fmt::Int16),
      2 => Ok(Fmt::Int32), 3 => Ok(Fmt::Int64),
      4 => Ok(Fmt::Float32), 5 => Ok(Fmt::Float64),
      _ => Err(err::RErr::Format)
    }
  }
}

/// Header of the RKPI2 format, it contains necessary
/// metadata to reproduce encapsulated audio data.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Hdr {
  /// Sampleformat used to code audio samples to bytedata.
  pub format: Fmt,

  /// Sampling rate of PCM audio. This controls the time-
  /// resolution of audio. Allowed ones are: 
  /// 
  /// - 192000
  /// - 96000
  /// - 64000
  /// - 44100
  /// - 32000
  /// - 22050
  /// - 12000
  /// - 8000
  pub rate: u32,

  /// Number of audio channels. The layout of channels is
  /// always interleaved.
  pub channels: u8
}