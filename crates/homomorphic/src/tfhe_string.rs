use crate::{HeError, HomomorphicEncryption};
use serde::{Deserialize, Serialize};
use tfhe::{ClearString, ClientKey, ConfigBuilder, FheStringIsEmpty, FheStringLen, generate_keys};
use tfhe::{FheAsciiString, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TfheString;

impl HomomorphicEncryption for TfheString {
    type SecretKey = ClientKey;
    type PublicKey = tfhe::ServerKey;
    type Ciphertext = FheAsciiString;
    type Plaintext = String;

    fn keygen() -> Result<(Self::SecretKey, Self::PublicKey), HeError> {
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        Ok((client_key, server_key))
    }

    fn encrypt(sk: &Self::SecretKey, pt: &Self::Plaintext) -> Result<Self::Ciphertext, HeError> {
        FheAsciiString::try_encrypt(pt, sk).map_err(|_| HeError::EncryptError)
    }

    fn decrypt(sk: &Self::SecretKey, ct: &Self::Ciphertext) -> Result<Self::Plaintext, HeError> {
        Ok(ct.decrypt(sk))
    }

    // No arithmetic for strings, so these are left unimplemented
    fn add(_: &Self::Ciphertext, _: &Self::Ciphertext) -> Result<Self::Ciphertext, HeError> {
        Err(HeError::UnsupportedOperation(
            "add not supported for strings".into(),
        ))
    }

    fn mul(_: &Self::Ciphertext, _: &Self::Ciphertext) -> Result<Self::Ciphertext, HeError> {
        Err(HeError::UnsupportedOperation(
            "mul not supported for strings".into(),
        ))
    }
}

impl TfheString {
    /// Compares two encrypted ASCII strings for equality.
    pub fn eq_enc(a: &FheAsciiString, b: &FheAsciiString) -> tfhe::FheBool {
        a.eq(b)
    }

    /// Compares two encrypted ASCII strings for inequality.
    pub fn ne_enc(a: &FheAsciiString, b: &FheAsciiString) -> tfhe::FheBool {
        a.ne(b)
    }

    /// Returns the encrypted length of the ASCII string.
    pub fn len_enc(s: &FheAsciiString) -> FheStringLen {
        s.len()
    }

    /// Returns whether the encrypted ASCII string is empty.
    pub fn is_empty_enc(s: &FheAsciiString) -> FheStringIsEmpty {
        s.is_empty()
    }

    /// Converts all encrypted characters to lowercase.
    pub fn to_lowercase_enc(s: &FheAsciiString) -> FheAsciiString {
        s.to_lowercase()
    }

    /// Converts all encrypted characters to uppercase.
    pub fn to_uppercase_enc(s: &FheAsciiString) -> FheAsciiString {
        s.to_uppercase()
    }

    /// Trims encrypted whitespace from both ends.
    pub fn trim_enc(s: &FheAsciiString) -> FheAsciiString {
        s.trim()
    }

    /// Trims encrypted whitespace from the start.
    pub fn trim_start_enc(s: &FheAsciiString) -> FheAsciiString {
        s.trim_start()
    }

    /// Trims encrypted whitespace from the end.
    pub fn trim_end_enc(s: &FheAsciiString) -> FheAsciiString {
        s.trim_end()
    }

    /// Removes a clear prefix from the encrypted string if present.
    pub fn strip_prefix_enc(
        s: &FheAsciiString,
        prefix: &ClearString,
    ) -> (FheAsciiString, tfhe::FheBool) {
        s.strip_prefix(prefix)
    }

    /// Removes a clear suffix from the encrypted string if present.
    pub fn strip_suffix_enc(
        s: &FheAsciiString,
        suffix: &ClearString,
    ) -> (FheAsciiString, tfhe::FheBool) {
        s.strip_suffix(suffix)
    }

    /// Decrypts an encrypted ASCII string.
    pub fn decrypt_string(ct: &FheAsciiString, ck: &ClientKey) -> String {
        ct.decrypt(ck)
    }

    /// Decrypts an encrypted boolean value.
    pub fn decrypt_bool(ct: &tfhe::FheBool, ck: &ClientKey) -> bool {
        ct.decrypt(ck)
    }

    /// Decrypts an encrypted string length.
    pub fn decrypt_len(len: &FheStringLen, ck: &ClientKey) -> u16 {
        match len {
            FheStringLen::NoPadding(l) => *l,
            FheStringLen::Padding(enc_l) => enc_l.decrypt(ck),
        }
    }

    /// Decrypts an encrypted “is empty” flag.
    pub fn decrypt_is_empty(flag: &FheStringIsEmpty, ck: &ClientKey) -> bool {
        match flag {
            FheStringIsEmpty::NoPadding(v) => *v,
            FheStringIsEmpty::Padding(enc_v) => enc_v.decrypt(ck),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use tfhe::set_server_key;

    #[test]
    fn test_encrypt_decrypt_string() {
        let (client_key, server_key) = TfheString::keygen().unwrap();
        set_server_key(server_key.clone());

        let value = String::from("Hello TFHE!");
        let ct = TfheString::encrypt(&client_key, &value).unwrap();
        let dec = TfheString::decrypt_string(&ct, &client_key);

        assert_eq!(dec, value);
    }

    #[test]
    fn test_string_eq_ne() {
        let (ck, sk) = TfheString::keygen().unwrap();
        set_server_key(sk);

        let s1 = FheAsciiString::try_encrypt("Zama", &ck).unwrap();
        let s2 = FheAsciiString::try_encrypt("zama", &ck).unwrap();

        let eq_false = TfheString::eq_enc(&s1, &s2);
        let eq_true = TfheString::eq_enc(&s1, &s1);
        let ne_true = TfheString::ne_enc(&s1, &s2);
        let ne_false = TfheString::ne_enc(&s1, &s1);

        assert!(!TfheString::decrypt_bool(&eq_false, &ck));
        assert!(TfheString::decrypt_bool(&eq_true, &ck));
        assert!(TfheString::decrypt_bool(&ne_true, &ck));
        assert!(!TfheString::decrypt_bool(&ne_false, &ck));
    }

    #[test]
    fn test_case_conversion() {
        let (ck, sk) = TfheString::keygen().unwrap();
        set_server_key(sk);

        let s = FheAsciiString::try_encrypt("TfHe123!", &ck).unwrap();
        let lower = TfheString::to_lowercase_enc(&s);
        let upper = TfheString::to_uppercase_enc(&s);

        assert_eq!(TfheString::decrypt_string(&lower, &ck), "tfhe123!");
        assert_eq!(TfheString::decrypt_string(&upper, &ck), "TFHE123!");
    }

    #[test]
    fn test_trim_and_strip() {
        let (ck, sk) = TfheString::keygen().unwrap();
        set_server_key(sk);

        let s = FheAsciiString::try_encrypt("   tfhe-rs   zama   ", &ck).unwrap();

        let trimmed_start = TfheString::trim_start_enc(&s);
        let trimmed_end = TfheString::trim_end_enc(&s);
        let trimmed = TfheString::trim_enc(&s);

        assert_eq!(
            TfheString::decrypt_string(&trimmed_start, &ck),
            "tfhe-rs   zama   "
        );
        assert_eq!(
            TfheString::decrypt_string(&trimmed_end, &ck),
            "   tfhe-rs   zama"
        );
        assert_eq!(TfheString::decrypt_string(&trimmed, &ck), "tfhe-rs   zama");

        let s = FheAsciiString::try_encrypt("The lazy cat", &ck).unwrap();

        let (res, ok) = TfheString::strip_prefix_enc(&s, &ClearString::new("The".into()));
        assert!(TfheString::decrypt_bool(&ok, &ck));
        assert_eq!(TfheString::decrypt_string(&res, &ck), " lazy cat");

        let (res, ok) = TfheString::strip_suffix_enc(&s, &ClearString::new("cat".into()));
        assert!(TfheString::decrypt_bool(&ok, &ck));
        assert_eq!(TfheString::decrypt_string(&res, &ck), "The lazy ");
    }
}
