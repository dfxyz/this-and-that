use std::io::{Read, Write};
use std::thread::spawn;

use aes_gcm::aes::cipher::Unsigned;
use aes_gcm::{AeadInPlace, Aes128Gcm, Aes256Gcm, KeyInit, Nonce};
use crossbeam_channel::{Receiver, Sender};
use rand::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};

const CHUNK_SIZE: usize = 4096;

struct CipherText {
    nonce: Vec<u8>,
    vec: Vec<u8>,
}
impl CipherText {
    fn len(&self) -> usize {
        self.nonce.len() + self.vec.len()
    }
}

pub fn encrypt<R, W, C, RNG>(
    reader: &mut R,
    writer: &mut W,
    cipher: &C,
    rng: &mut RNG,
) -> Result<(), Vec<String>>
where
    R: Read,
    W: Write,
    C: AeadInPlace,
    RNG: CryptoRng + RngCore,
{
    let (plaintext_tx, plaintext_rx) = crossbeam_channel::unbounded();
    let (ciphertext_tx, ciphertext_rx) = crossbeam_channel::unbounded();

    let ptr = reader as *mut R as usize;
    let h1 = spawn(move || {
        let reader = unsafe { (ptr as *mut R).as_mut() }.unwrap();
        produce_plaintext(reader, plaintext_tx)
    });

    let ptr1 = cipher as *const C as usize;
    let ptr2 = rng as *mut RNG as usize;
    let h2 = spawn(move || {
        let cipher = unsafe { (ptr1 as *const C).as_ref() }.unwrap();
        let rng = unsafe { (ptr2 as *mut RNG).as_mut() }.unwrap();
        do_encrypt(cipher, rng, plaintext_rx, ciphertext_tx)
    });

    let ptr = writer as *mut W as usize;
    let h3 = spawn(move || {
        let writer = unsafe { (ptr as *mut W).as_mut() }.unwrap();
        consume_ciphertext(writer, ciphertext_rx)
    });

    let r1 = h1.join().unwrap();
    let r2 = h2.join().unwrap();
    let r3 = h3.join().unwrap();

    let mut msg_vec = vec![];
    match r1 {
        Ok(_) => {}
        Err(msg) => {
            msg_vec.push(msg);
        }
    }
    match r2 {
        Ok(_) => {}
        Err(msg) => {
            msg_vec.push(msg);
        }
    }
    match r3 {
        Ok(_) => {}
        Err(msg) => {
            msg_vec.push(msg);
        }
    }
    if msg_vec.is_empty() {
        Ok(())
    } else {
        Err(msg_vec)
    }
}

fn produce_plaintext<R: Read>(reader: &mut R, plaintext_tx: Sender<Vec<u8>>) -> Result<(), String> {
    loop {
        let mut buf = vec![0u8; CHUNK_SIZE];
        match reader.read(&mut buf) {
            Ok(len) => {
                buf.truncate(len);
            }
            Err(e) => {
                let msg = format!("read error: {e}");
                return Err(msg);
            }
        }
        let eof = buf.is_empty();
        match plaintext_tx.send(buf) {
            Ok(_) => {
                if eof {
                    return Ok(());
                }
            }
            Err(_) => {
                return Ok(()); // encryptor stopped; error occurred
            }
        }
    }
}

fn do_encrypt<C: AeadInPlace, RNG: CryptoRng + RngCore>(
    cipher: &C,
    rng: &mut RNG,
    plaintext_rx: Receiver<Vec<u8>>,
    ciphertext_tx: Sender<Option<CipherText>>,
) -> Result<(), String> {
    loop {
        let mut buf = match plaintext_rx.recv() {
            Ok(b) => b,
            Err(_) => {
                return Ok(()); // producer stopped; error occurred
            }
        };

        if buf.is_empty() {
            let _ = ciphertext_tx.send(None); // indicates consumer about eof
            return Ok(());
        }

        let nonce = C::generate_nonce(&mut *rng);
        match cipher.encrypt_in_place(&nonce, b"", &mut buf) {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("encrypt error: {e}");
                return Err(msg);
            }
        }

        match ciphertext_tx.send(Some(CipherText {
            nonce: nonce.to_vec(),
            vec: buf,
        })) {
            Ok(_) => {}
            Err(_) => {
                return Ok(()); // consumer stopped; error occurred
            }
        }
    }
}

fn consume_ciphertext<W: Write>(
    writer: &mut W,
    ciphertext_rx: Receiver<Option<CipherText>>,
) -> Result<(), String> {
    loop {
        let ciphertext = match ciphertext_rx.recv() {
            Ok(b) => b,
            Err(_) => {
                return Ok(()); // encryptor stopped; error occurred
            }
        };

        let ciphertext = match ciphertext {
            None => {
                return Ok(());
            }
            Some(c) => c,
        };

        let len = u32::try_from(ciphertext.len()).unwrap().to_le_bytes();
        match writer.write_all(&len) {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("write error: {e}");
                return Err(msg);
            }
        }
        match writer.write_all(&ciphertext.nonce) {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("write error: {e}");
                return Err(msg);
            }
        }
        match writer.write_all(&ciphertext.vec) {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("write error: {e}");
                return Err(msg);
            }
        }
    }
}

pub fn decrypt<R, W, C>(reader: &mut R, writer: &mut W, cipher: &C) -> Result<(), Vec<String>>
where
    R: Read,
    W: Write,
    C: AeadInPlace,
{
    let (ciphertext_tx, ciphertext_rx) = crossbeam_channel::unbounded();
    let (plaintext_tx, plaintext_rx) = crossbeam_channel::unbounded();

    let ptr = reader as *mut R as usize;
    let nonce_size = C::NonceSize::to_usize();
    let tag_size = C::TagSize::to_usize();
    let h1 = spawn(move || {
        let reader = unsafe { (ptr as *mut R).as_mut() }.unwrap();
        produce_ciphertext(reader, ciphertext_tx, nonce_size, tag_size)
    });

    let ptr = cipher as *const C as usize;
    let h2 = spawn(move || {
        let cipher = unsafe { (ptr as *const C).as_ref() }.unwrap();
        do_decrypt(cipher, ciphertext_rx, plaintext_tx)
    });

    let ptr = writer as *mut W as usize;
    let h3 = spawn(move || {
        let writer = unsafe { (ptr as *mut W).as_mut() }.unwrap();
        consume_plaintext(writer, plaintext_rx)
    });

    let r1 = h1.join().unwrap();
    let r2 = h2.join().unwrap();
    let r3 = h3.join().unwrap();

    let mut msg_vec = vec![];
    match r1 {
        Ok(_) => {}
        Err(msg) => {
            msg_vec.push(msg);
        }
    }
    match r2 {
        Ok(_) => {}
        Err(msg) => {
            msg_vec.push(msg);
        }
    }
    match r3 {
        Ok(_) => {}
        Err(msg) => {
            msg_vec.push(msg);
        }
    }
    if msg_vec.is_empty() {
        Ok(())
    } else {
        Err(msg_vec)
    }
}

fn produce_ciphertext<R: Read>(
    reader: &mut R,
    ciphertext_tx: Sender<Option<CipherText>>,
    nonce_size: usize,
    tag_size: usize,
) -> Result<(), String> {
    loop {
        let mut len_buf = vec![0u8; 4];
        let len = match reader.read(&mut len_buf) {
            Ok(len) => {
                if len == 0 {
                    let _ = ciphertext_tx.send(None);
                    return Ok(());
                }
                if len != 4 {
                    let msg = format!("read error, illegal length {len}");
                    return Err(msg);
                }
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&len_buf);
                (u32::from_le_bytes(bytes)) as usize
            }
            Err(e) => {
                let msg = format!("read error: {e}");
                return Err(msg);
            }
        };

        assert!(len > tag_size + nonce_size);
        let mut nonce = vec![0u8; nonce_size];
        match reader.read_exact(&mut nonce) {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("read error: {e}");
                return Err(msg);
            }
        }
        let mut buf = vec![0u8; len - nonce_size];
        match reader.read_exact(&mut buf) {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("read error: {e}");
                return Err(msg);
            }
        }

        match ciphertext_tx.send(Some(CipherText { nonce, vec: buf })) {
            Ok(_) => {}
            Err(_) => {
                return Ok(()); // decryptor stopped; error occurred
            }
        }
    }
}

fn do_decrypt<C: AeadInPlace>(
    cipher: &C,
    ciphertext_rx: Receiver<Option<CipherText>>,
    plaintext_tx: Sender<Vec<u8>>,
) -> Result<(), String> {
    loop {
        let ciphertext = match ciphertext_rx.recv() {
            Ok(c) => c,
            Err(_) => {
                return Ok(()); // producer stopped; error occurred
            }
        };

        let mut ciphertext = match ciphertext {
            None => {
                let _ = plaintext_tx.send(vec![]); // indicates consumer about eof
                return Ok(());
            }
            Some(c) => c,
        };

        match cipher.decrypt_in_place(
            Nonce::from_slice(&ciphertext.nonce),
            b"",
            &mut ciphertext.vec,
        ) {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("decrypt error: {e}");
                return Err(msg);
            }
        }

        match plaintext_tx.send(ciphertext.vec) {
            Ok(_) => {}
            Err(_) => {
                return Ok(()); // consumer stopped; error occurred
            }
        }
    }
}

fn consume_plaintext<W: Write>(
    writer: &mut W,
    plaintext_rx: Receiver<Vec<u8>>,
) -> Result<(), String> {
    loop {
        let buf = match plaintext_rx.recv() {
            Ok(b) => b,
            Err(_) => {
                return Ok(()); // decryptor stopped; error occurred
            }
        };

        if buf.is_empty() {
            return Ok(());
        }

        match writer.write_all(&buf) {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("write error: {e}");
                return Err(msg);
            }
        }
    }
}

pub fn new_aes128gcm_cipher(key: &str) -> Aes128Gcm {
    let hash = Sha256::digest(key);
    Aes128Gcm::new_from_slice(&hash[..16]).unwrap()
}

pub fn new_aes256gcm_cipher(key: &str) -> Aes256Gcm {
    let hash = Sha256::digest(key);
    Aes256Gcm::new_from_slice(&hash).unwrap()
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{new_aes128gcm_cipher, decrypt, encrypt};

    #[test]
    fn test() {
        let cipher = new_aes128gcm_cipher("test");
        let mut rng = rand::thread_rng();

        let mut raw_bytes = vec![0u8; 4096 * 1024];
        for b in raw_bytes.iter_mut() {
            *b = rand::random();
        }
        let mut cursor = Cursor::new(&raw_bytes);

        let mut encrypted = vec![];
        encrypt(&mut cursor, &mut encrypted, &cipher, &mut rng).unwrap();

        let mut cursor = Cursor::new(&encrypted);
        let mut decrypted = vec![];
        decrypt(&mut cursor, &mut decrypted, &cipher).unwrap();

        raw_bytes.iter().enumerate().for_each(|(i, v)| {
            assert_eq!(*v, decrypted[i]);
        });
        assert_eq!(raw_bytes.len(), decrypted.len());
    }
}
