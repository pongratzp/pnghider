use crate::prelude::*;

mod error;
mod prelude;

mod utils;

use crate::utils::common::{find_sequence, Pngchunk};

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit},
    Aes256Gcm,
};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2,
};
use clap::Parser;
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
#[derive(Parser, Debug)]
struct Cli {
    key: String,
    infile: std::path::PathBuf,
    payload: std::path::PathBuf,
    outfile: std::path::PathBuf,
}

impl fmt::Display for Cli {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Key: {:?} Infile: {:?}, Payload: {:?}, Outfile: {:?}",
            self.key, self.infile, self.payload, self.outfile
        )
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();

    println!("CLI: {}", args);

    let f = File::open(args.infile)?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)?;

    match find_sequence(&buffer, &utils::common::PNG_MAGICBYTES) {
        Some(0) => (),
        Some(_) => {
            Error::Generic(f!("Magic bytes in weird position"));
        }
        None => {
            Error::Generic(f!("Could not read magic bytes"));
        }
    }

    let dstart = find_sequence(&buffer, &utils::common::PNG_IEND);
    match dstart {
        Some(n) => println!("Image END tag starts at byte {:?}", n),
        None => {
            Error::Generic(f!("Could not read IEND bytes"));
        }
    }

    let mut endchunk: Pngchunk = Default::default();
    let dstart = dstart.unwrap();
    endchunk.load_from_slice(buffer[dstart - utils::common::CHUNK_LEN_SIZE..].to_vec());

    let dlen = endchunk.chunk_len();

    println!("Got {:?} bytes of IEND", dlen);

    let sf = File::open(args.payload)?;
    let mut sreader = BufReader::new(sf);
    let mut sbuffer = Vec::new();

    sreader.read_to_end(&mut sbuffer)?;

    let mut aes_key = [0u8; 32];
    let binding = SaltString::generate(&mut OsRng);
    let mut saltbytes: [u8; utils::common::SALT_LEN] = [0u8; utils::common::SALT_LEN];
    let _ = binding.decode_b64(&mut saltbytes);

    let _ = Argon2::default().hash_password_into(args.key.as_bytes(), &saltbytes, &mut aes_key);

    let cipher = Aes256Gcm::new_from_slice(&aes_key).unwrap();
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    println!("nonce: {:?}", nonce);
    println!("salt: {:?}", saltbytes);

    let mut ciphertext = cipher
        .encrypt(&nonce, sbuffer.as_ref())
        .map_err(|err| Error::Generic(f!("could not encrypt: {:?}", err.to_string())))?;

    ciphertext.splice(0..0, saltbytes);
    ciphertext.splice(0..0, nonce);

    let mut newchunk: Pngchunk = Default::default();
    newchunk.create_from_content(utils::common::PNG_CUSTOMCHUNK, ciphertext);

    let i = dstart + utils::common::CHUNK_HEADER_SIZE + dlen + utils::common::CHUNK_CRC_SIZE;
    buffer.splice(i..i, newchunk.flatten());

    let endtag: [u8; 12] = [0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130];
    buffer.extend(endtag);

    let o = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(args.outfile)?;
    let mut writer = BufWriter::new(o);

    writer.write_all(&mut buffer)?;

    Ok(())
}
