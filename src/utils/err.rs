#[derive(Debug)]
pub enum RErr {
  /// Invalid start-code for identification
  /// of a valid Reekpie file.
  StartCode,
  /// Sample-format was reserved for future usage
  /// but still used in the format.
  Format,
  /// Error occured while doing I/O with given writer
  /// or passing it to compressor/decompressor.
  IO,
  /// Input samplerate was not valid for RKPI2.
  Rate,
  /// Number of input channels was not valid.
  Channels
}