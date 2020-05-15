//! Pure Rust implementation of the RKPI2 audio format.
//! This format is aimed to replace WAVE with a simple
//! to parse an minimal header strutcture, ability to
//! easily parse and optional compression with Zstd.
//! 
//! This can both mux and demux the header data, from
//! file objects and has a simple interface.
//! 
//! # Example
//! ```
//! use std::io::Cursor;
//!
//! fn main() {
//!     let out = Cursor::new(Vec::new());
//!     let mut rkout = mux(Box::new(out),
//!         Hdr {
//!             format: Fmt::Int8,
//!             rate: 8000,
//!             channels: 1
//!         }, None).unwrap();
//!     rkout.write_all(vec![0u8; 8000].as_slice()).unwrap();
//! }
//! ```

use std::convert::TryFrom;
use std::io::{Read, Write};
use zstd::{Encoder, Decoder};

mod utils;
pub use utils::{Fmt, Hdr, RErr};

/// A defined set of samplerates allowed for the PCM
/// data encapsulated inside RKPI2.
const SAMPLERATES: [u32; 8] = [
    8000, 12000, 22050, 32000, 44100,
    64000, 96000, 192000 ];

/// Mux RKPI2 header data into a writer so decoders can
/// decode the PCM data.
/// 
/// # Arguments
/// * `w` — boxed writer to write in RKPI2 header data.
/// * `h` — header to serialise as of specification and write.
/// * `lev` — level of Zstd compression ranged (1..+21].
fn mux(w: Box<dyn Write>, h: Hdr, lev: Option<u8>)
    -> Result<Box<dyn Write>, RErr> {
    let mut w = w;

    let srate_idx = match SAMPLERATES
        .iter().position(|&s| s == h.rate) {
        Some(S) => S as u8,
        None    => { return Err(RErr::Rate) }
    };

    let channels = match h.channels {
        (1..=8) => h.channels,
        _       => { return Err(RErr::Channels) }
    };

    let compressed = match lev {
        Some(_) => true, None => false };

    if let Err(_) = w.write_all(&[
        0x3d                 << 2|
        (compressed as u8)   << 1|
        (h.format as u8)     >> 2,
        (h.format as u8 & 3) << 6|
        srate_idx            << 3|
        channels        - 1
    ]) { return Err(RErr::IO) }

    // if compression was an option wrap with the Zstd stream encoder
    // else just return the same writer back for writing data.
    match lev {
        Some(L) => match Encoder::new(w, L as i32)
        { Ok(C)  => Ok(Box::new(C)),
          Err(_) => Err(RErr::IO) },
        None    => Ok(w)
    }
}

/// Demux RKPI2 header data from the given reader, if compression
/// was done before it wraps reader with Zstd decompressor.
/// 
/// # Arguments
/// * `r` — boxed reader to parse RKPI2 header data.
fn demux(r: Box<dyn Read>)
    -> Result<(Box<dyn Read>, Hdr), RErr> {
    let mut r = r;

    let mut hdr = [0u8; 2];
    if let Err(_) = r.read(&mut hdr)
        { return Err(RErr::IO); }

    if hdr[0] >> 2 != 0x3d { Err(RErr::StartCode) }
    else {
        let format = match Fmt::try_from(
            (hdr[0] & 1) << 2 | hdr[1] >> 6)
        { Ok(F) => F, Err(E) => { return Err(E) } };

        let h = Hdr {
            format: format,
            rate: SAMPLERATES[(hdr[1] >> 3 & 3) as usize],
            channels: (hdr[1] & 3) + 1
        };

        // if decompression is a requirement, wrap it up with Zstd decompressor.
        match (hdr[0] >> 1 & 1) == 1 {
            true  => match Decoder::new(r)
            { Ok(D)  => Ok((Box::new(D), h)),
              Err(_) => Err(RErr::IO) },
            false => Ok((r, h))
        }
    }
}

#[test]
fn rkpi2_hdr_and_data() {
  // use a pipe for redirecting the output got by RKPI2
  // muxer to an input stream where we demux it.
  use os_pipe::pipe;

  // use the most minimalist configuration, for the least
  // cpu and ram load.
  let ihdr = Hdr { format: Fmt::Int8,
                   rate: 8000,
                   channels: 1 };
  
  let isamples = vec![127u8; ihdr.rate as usize 
                             * ihdr.channels as usize];
  let mut osamples = vec![0u8; ihdr.rate as usize
                             * ihdr.channels as usize];

  let (inp, out) = pipe().unwrap();

  // use zstd with least power available to it, and also test
  // the compression method works correctly and produces
  // accurate data provided to it, though it's not guranteed
  // that this would always work.
 
  let mut rkout = mux(Box::new(out), ihdr, Some(1)).unwrap();
  rkout.write(&isamples).unwrap();
  rkout.flush().unwrap();

  let (mut rkin, ohdr) = demux(Box::new(inp)).unwrap();
  rkin.read(&mut osamples).unwrap();

  assert!(ihdr == ohdr);
  assert!(isamples == osamples);
}