mod error;
mod prelude;

mod utils;

use crate::utils::common::{check_png, get_chunk_start, Pngchunk};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};

use argon2::Argon2;
use clap::Parser;
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
#[derive(Parser)]
struct Cli {
    key: String,
    infile: std::path::PathBuf,
    outfile: std::path::PathBuf,
}

impl fmt::Display for Cli {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Key: {:?} Infile: {:?}, Outfile: {:?}",
            self.key, self.infile, self.outfile
        )
    }
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    println!("CLI: {}", args);

    let f = File::open(args.infile)?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)?;

    check_png(&buffer)?;

    let dstart = get_chunk_start(&buffer, &utils::common::PNG_CUSTOMCHUNK)?;

    let mut encoded_chunk: Pngchunk = Default::default();

    encoded_chunk.load_from_slice(buffer[dstart - utils::common::CHUNK_LEN_SIZE..].to_vec());
    let dlen = encoded_chunk.chunk_len();

    println!("Got {:?} bytes of encoded data!", dlen);

    let encdata = encoded_chunk.content();
    let saltbytes =
        &encdata[utils::common::NONCE_LEN..utils::common::NONCE_LEN + utils::common::SALT_LEN];
    let nonce = Nonce::from_slice(&encdata[0..utils::common::NONCE_LEN]);

    let mut aes_key = [0u8; 32];

    let _ = Argon2::default().hash_password_into(args.key.as_bytes(), &saltbytes, &mut aes_key);

    let cipher = Aes256Gcm::new_from_slice(&aes_key).unwrap();
    println!("nonce: {:?}", nonce);
    println!("salt: {:?}", saltbytes);

    let plaintext = cipher
        .decrypt(
            &nonce,
            encdata[utils::common::NONCE_LEN + utils::common::SALT_LEN..].as_ref(),
        )
        .map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("could not decrypt: {:?}", err.to_string()),
            )
        })?;

    let o = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(args.outfile)?;
    let mut writer = BufWriter::new(o);

    writer.write_all(&plaintext)?;

    Ok(())
}
