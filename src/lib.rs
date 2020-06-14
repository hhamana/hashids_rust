//! Obfuscate ID into Hashids, for public, polite, and less predictible identifiers.
//! 
//! Hashid allows short, seemingly random, ids. They are unique by using a secret salt.
//! 
//! Principle of this library:
//! Use the [HashidBuilder](struct.HashidBuilder) to configure, then use the returned [codec](struct.HashidCodec) to encode and decode IDs.
//! 
//! Features of this crate over other crates on crates.io:
//! - Convenient, Rust-friendly API
//! - Lazy performance hacks to prentend it's fast
//! - An inconsistent amount of documentation to make sure you are confused.
//! - Returns so many Errors for your pleasure to handle 
//! - Integration with serde and diesel, "coming soon"
use std::collections::{HashSet};
use regex::Regex;

const ENV_KEY: &'static str = "HASHID_SALT";
const DEFAULT_ALPHABET: &'static str =  "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
const DEFAULT_MIN_LENGTH : usize = 4;
const DEFAULT_SEPARATORS: &'static str = "cfhistuCFHISTU";
const SEPARATOR_DIV: f32 = 3.5;
const GUARD_DIV: usize = 12;
const MIN_ALPHABET_LENGTH: usize = 16;

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
  /// A unique salt must be provided when building the HashidCodec. There are two ways to do so: 
  /// - using either or the `with_salt`, `with_string_salt`, `with_hashid_salt` API
  /// - setting a `HASHID_SALT` environnment variable.
  /// A salt is just a string, that has to be provided to provide a unique (compared to other packages using the same hashing algorithm)
  /// and repeatable (it must not change, so the encoding and decoding of a string/integer yields the same result.)
  MissingSalt,
  NonAsciiSalt,
  InvalidAlphabetLength,
  NonAsciiAlphabet,
  InvalidInputId,
  NonHexString,
  EmptyHash,
  InvalidHash
}

/// Represents the salt to use when encoding/decoding IDs.
/// 
/// It is of course recommended to keep that value in an environnment variable.
/// By default it will use the environnment variable called `HASHID_SALT`.
/// 
/// This struct can be publicly used for manual hardcoded construction, or created dynamically from a String.
// It also doesn't need to be String, a &str is enough, as the salt is likely to be hardcoded anyway.
// 
/// There is no default, it will return a hashid::Error::MissingSalt if it cannot be created.
#[derive(Debug, PartialEq)]
pub struct HashidSalt(String);

impl From<&str> for HashidSalt {
  fn from(s: &str) -> HashidSalt {
    HashidSalt(s.to_string())
  }
}

impl From<String> for HashidSalt {
  fn from(s: String) -> HashidSalt {
    HashidSalt(s)
  }
}

  /// Use this builder to setup the hashid encoder/decoder [HashidCodec](struct.HashidCodec.html).
  /// 
  /// There are many options to customize the encoder, and by extension, hashing settings, 
  /// but it should be most straightforward to build if you have the `HASHID_SALT` environnment variable set up.
  /// It defaults to a minimum length for hashes of 4 digits, and an alphabet with lowercase, uppercase and numbers 0 to 9.
  /// With environnment variable `HASHID_SALT` and just using the default configuration, the code to create the encoder/decoder is as follows:
  /// 
  /// ```
  /// # std::env::set_var("HASHID_SALT", "organic salt");
  /// use hashids::{HashidBuilder};
  /// let builder = HashidBuilder::new().ok().expect("Failed building the hashid encoder");
  /// ```
  /// Use the `with_` methods to configure custom settings through code, 
  /// and finish with the `.ok()` to get a Result containing the HashidCoDec with your configuration.
  /// ```
  /// use hashids::{HashidBuilder};
  /// let builder_result = HashidBuilder::new()
  ///       .with_salt("my salt")
  ///       .with_alphabet("12345789abcedef~!@#$%^&*()_+".to_string())
  ///       .with_length(16)
  ///       .ok().unwrap();
  /// ```
pub struct HashidBuilder {
  salt: Option<HashidSalt>,
  alphabet: Option<String>,
  min_length: Option<usize>
}

impl HashidBuilder {
  pub fn new() -> HashidBuilder {
    HashidBuilder {
      salt: None,
      alphabet: None,
      min_length: None
    }
  }

  // Salt-related methods
  /// Allows you to create the HashidSalt separately, and use it in the builder.
  /// `with_salt()` should be more convenient as it does this steps internally.
  /// ```
  /// use hashids::{HashidSalt, HashidBuilder};
  /// let salt = HashidSalt::from("my_custom salt");
  /// let builder_result = HashidBuilder::new().with_hashid_salt(salt).ok();
  /// ```
  pub fn with_hashid_salt(mut self, salt: HashidSalt) -> HashidBuilder {
    self.salt = Some(salt);
    self
  }
  
  /// Creates a salt from a &str. Use this method to create a `HashidBuilder` with a custom hardcoded salt.
  /// This should be the most convenient method to experiment with the library.
  /// You can consider using environnment variables instead. 
  /// The builder will use the `HASHID_SALT` environnment variable to build salt if it isn't defined with code.
  /// ```
  /// use hashids::{HashidBuilder};
  /// let builder_result = HashidBuilder::new().with_salt("my salt").ok();
  /// ```
  pub fn with_salt(self, salt: &str) -> HashidBuilder {
    let hashid_salt = HashidSalt::from(salt);
    self.with_hashid_salt(hashid_salt)
  }
  
  /// Creates a salt from a String. Use this method to create a `HashidBuilder` with a custom hardcoded salt.
  /// Rather than String, &str should be easier to construct, you can use `with_salt` instead.
  /// You can consider using environnment variables instead. 
  /// The builder will use the `HASHID_SALT` environnment variable to build salt if it isn't defined with code.
  /// ```
  /// use hashids::{HashidBuilder};
  /// let string_salt = "my salt".to_string();
  /// let builder_result = HashidBuilder::new().with_string_salt(string_salt).ok();
  /// ```
  pub fn with_string_salt(self, salt: String) -> HashidBuilder {
    let hashid_salt = HashidSalt::from(salt);
    self.with_hashid_salt(hashid_salt)
  }
  
  
  // Alphabet-related methods
  /// Add a custom alphabet. The default alphabet is "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".
  /// Must be greater than 16 symbols long. 
  /// Characters cannot be more than 1 byte length, so only ASCII characters are allowed.
  /// ```
  /// use hashids::{HashidBuilder, Error};
  /// let builder = HashidBuilder::new()
  ///     .with_salt("漢字注入注意")
  ///     .with_alphabet("あいうえおかきくけこたちつてとさしすせそ".to_string())
  ///     .ok();
  /// assert_eq!(builder, Err(Error::NonAsciiAlphabet));
  /// ```
  pub fn with_alphabet(mut self, alphabet: String) -> HashidBuilder {
    self.alphabet = Some(alphabet); 
    self
  }

  /// Adjust the length of the hash string to be generated.
  pub fn with_length(mut self, length: usize) -> HashidBuilder {
    self.min_length = Some(length);
    self
  }

  /// Creates an complete instance of HashidCodec, validating it settings.
  /// Errors if incomplete in crucial parts.
  /// The builder returned can then be used to encode and decode.
  /// Can return an `Error::MissingSalt` if no salt is set (through environnment variable or its API),
  /// or a `Error::InvalidAlphabetLength` if a custom alphabet is shorter than 16 characters.
  ///  ```
  /// use hashids::{HashidBuilder};
  /// let builder_result = HashidBuilder::new().ok();
  /// ```
  pub fn ok(self) -> Result<HashidCodec, Error>  {

    // Get custom alphabet or default otherwise
    let alphabet = {
      match self.alphabet {
        // Default alphabet is already manually checked to be only unique ascii chars, no need to revalidate that
        None => DEFAULT_ALPHABET.to_string(),
        Some(custom) => {
          if !custom.is_ascii() { return  Err(Error::NonAsciiAlphabet ) }
          let unique = get_unique_alphabet(custom);
          if unique.len() < MIN_ALPHABET_LENGTH { return Err(Error::InvalidAlphabetLength) };
          unique
        }
      }
    };
    // get custom salt, set from builder function or by environnment
    let salt = if let Some(custom) = self.salt { if !custom.0.is_ascii() { return  Err(Error::NonAsciiSalt ) } custom } else { 
      let by_env = std::env::var(ENV_KEY);
      match by_env {
        Ok(var) => HashidSalt::from(var),
        Err(_) => return Err(Error::MissingSalt)
      }
    };
    
    let min_hash_length = if let Some(custom) = self.min_length { custom } else { DEFAULT_MIN_LENGTH };
    
    let (t_separators, mut t_alphabet) = get_non_duplicated_string(DEFAULT_SEPARATORS.to_string(), alphabet);
    let mut shuffled_separators = hashids_shuffle(t_separators.clone(), &salt)?;
    let alphabet_len = t_alphabet.len();
    
    let shuffled_separators_len = shuffled_separators.len();

    if shuffled_separators_len <= 0 || ((alphabet_len/shuffled_separators_len) as f32) > SEPARATOR_DIV {
      let mut seps_len =  ((alphabet_len as f32) / SEPARATOR_DIV) as usize;
      if seps_len == 1 {
        seps_len = 2;
      };

      if seps_len > shuffled_separators_len {
        let diff = seps_len - shuffled_separators_len;

        shuffled_separators.push_str(&t_alphabet[..diff]);
        t_alphabet = t_alphabet[diff..].to_string();
      } else {
        shuffled_separators = shuffled_separators[..seps_len].to_string();
      };
    };

    let mut shuffled_alphabet = hashids_shuffle(t_alphabet, &salt)?;

    let guard_count = (alphabet_len as f32 / GUARD_DIV as f32).ceil() as usize;

    let t_guards;

    if alphabet_len < 3 {
      t_guards = shuffled_separators[..guard_count].to_string();
      shuffled_separators = shuffled_separators[guard_count..].to_string();
    } else {
      t_guards = shuffled_alphabet[..guard_count].to_string();
      shuffled_alphabet = shuffled_alphabet[guard_count..].to_string();
    };

    Ok(HashidCodec {
      salt,
      min_hash_length,
      guards: t_guards,
      separators: shuffled_separators,
      alphabet: shuffled_alphabet
    })
  }
}

/// This struct manages encoding and decoding according to the validated alphabet and salt.
///
/// It can only be created from a `HashidBuilder`, to validate and process input values conveniently.
/// Once created, you can use the `.encode()` and `.decode` methods.
#[derive(Debug, PartialEq)]
pub struct HashidCodec {
  salt: HashidSalt,
  alphabet: String,
  separators: String,
  min_hash_length: usize,
  guards: String 
}

/// Uses a `HashidBuilder::new().ok()` and panics in case of error, which means it must have a salt set through environnment variables.
/// 
/// HashidCodec having a default implementation available does not mean it would be wise to skip building a lasting one and generate a new builder when needed,
/// because the creation process involves some validation that can be heavy if repeated needlessly.  
/// Having a persistent object keeping the settings avoids this performance hit.
impl Default for HashidCodec {
  fn default() -> Self {
      match HashidBuilder::new().ok() {
        Ok(codec) => codec,
        Err(err) => {
          match err {
            Error::MissingSalt => panic!("HashidCodec default implementation relies on the 'HASHID_SALT' environnment variable being set"),
            _ => panic!("Unexpected failure to build the HashidCodec through the HashidBuilder defaults."),
          }
        }
      }
  }
}

impl HashidCodec {

  // TODO: investigate if I even need this.
  // pub fn decode_hex(&self, hash: String) -> String {
  //   let numbers = self.decode(hash);

  //   // TODO: I get the feeling this is something stupid oveengineered.
  //   let mut ret = String::new();
  //   for number in numbers {
  //     let r = format!("{:x}", number);
  //     ret.push_str(&r[1..]);
  //   }

  //   ret
  // }

  /// Converts an ID integer to a Hashid String.
  ///
  /// The integer can be any PositiveInteger (u32, u64, i32 and i64 are included), valid from 0 to 9007199254740992. (i64 max).
  /// The trait PositiveInteger must be in scope to allow generic usage.
  /// ```
  /// use hashids::{HashidBuilder, PositiveInteger, HashidCodec};
  /// let codec = HashidBuilder::new().with_salt("this is my salt").ok().unwrap();
  /// let encoded_id = codec.encode(5i64).unwrap();
  /// assert_eq!( encoded_id, "0rDd".to_string() );
  ///
  /// let negative_id = codec.encode(-2);
  /// assert_eq!( negative_id, Err(hashids::Error::InvalidInputId) );
  /// ```
  ///
  /// Why allow i64? It could be possible to erase the possibility of seeing negative numbers by just accepting usize, u32 and u64.
  /// However, the main usage of hashid is to obfuscate DB ids, and considering the prevalent use of diesel in the Rust ecosystem, it only makes sense to allow convenient interfacing.  
  /// Diesel converts database ids to i64. Thereforce, they are are allowed and checked to be positive at runtime.
  ///
  /// Why are negative numbers disallowed?  
  /// The hashid algorithm works through indexing in the alphabet, salt, and some guards characters, and a negative would throw the indexing and calculations off.
  pub fn encode<T: PositiveInteger>(&self, id: T) -> Result<String, Error> {
    // Validate/Convert Input as a positive i64. 
    // Error depending on PositiveInteger implementation, but probably a Error::InvalidInputId
    let as_usize = id.to_usize()?;

    // TODO ?: make it not needing to be a vec, even internally?
    let numbers = vec![as_usize];
    let id = self.encode_vec(&numbers);
    Ok(id)
  }

  fn encode_vec(&self, numbers: &Vec<usize>) -> String {
    let mut number_hash_int  = 0;
    
    // magic number
    let mut count = 100; 

    for number in numbers.iter() {
      number_hash_int += number % count;
      count += 1;
    };

    let idx = number_hash_int % self.alphabet.len();
    let ret = self.alphabet[idx..idx+1].to_string();
    let mut ret_str = ret.clone();

    let mut t_alphabet = self.alphabet.clone();
    let mut i = 0;
    let len = self.separators.len();
    let last_len = numbers.len();
    for number in numbers.iter() {
      let buffer = format!("{}{}{}", ret, self.salt.0, t_alphabet);
      t_alphabet = hashids_shuffle(t_alphabet.clone(), &HashidSalt::from(&buffer[0..t_alphabet.len()])).unwrap();
      let last = hash(*number, &t_alphabet);

      ret_str.push_str(&last);

      if (i + 1) < last_len {
        let mut v = *number % (last.as_bytes()[0] as usize + i as usize);
        v = v % len;
        ret_str.push(self.separators.as_bytes()[v as usize] as char);
      }
      i += 1;
    };

    if ret_str.len() < self.min_hash_length {
      let guard_idx = (number_hash_int + ret_str.clone().into_bytes()[0] as usize) % self.guards.len();
      let guard = self.guards[guard_idx..guard_idx+1].to_string();
      // let mut t = guard.clone();
      // t.push_str(&ret_str);
      ret_str = format!("{}{}", guard, ret_str);

      if ret_str.len() < self.min_hash_length {
        let guard_idx = (number_hash_int + ret_str.clone().into_bytes()[2] as usize) % self.guards.len();
        ret_str.push_str(&self.guards[guard_idx..guard_idx+1]);
      }
    };

    let half_len = t_alphabet.len() / 2;
    while ret_str.len() < self.min_hash_length {
      t_alphabet = hashids_shuffle(t_alphabet.clone(), &HashidSalt::from(t_alphabet)).unwrap();
      let mut t_ret = "".to_string();
      t_ret.push_str(&t_alphabet[half_len..]);
      t_ret.push_str(&ret_str[..]);
      t_ret.push_str(&t_alphabet[0..half_len]);
      ret_str = t_ret;

      let excess = ret_str.len() - self.min_hash_length;
      if excess > 0 {
        let start_pos = excess / 2;
        ret_str = ret_str[start_pos..start_pos + self.min_hash_length].to_string();
      }
    };

    ret_str
  }

  pub fn decode(&self, hash: String) -> Result<Vec<usize>, Error> {
    if hash.is_empty() {
      return Err(Error::EmptyHash)
    }
    
    let regexp = format!("[{}]", self.guards);
    let re = Regex::new(&regexp).unwrap();
    let t_hash = re.replace_all(&hash, " ");
    let split1: Vec<&str> = t_hash.split_whitespace().collect();

    let mut i = 0;

    let len = split1.len();
    if len == 3 || len == 2 {
      i = 1;
    }
    let mut hash_breakdown = split1[i].to_string();

    let lottery = hash_breakdown[0..1].to_string();
    hash_breakdown = hash_breakdown[1..].to_string();

    let regexp2 = format!("[{}]", self.separators);
    let re2 = Regex::new(&regexp2).unwrap();
    hash_breakdown = re2.replace_all(&hash_breakdown, " ").to_string();
    let split2: Vec<&str> = hash_breakdown.split_whitespace().collect();

    let mut alphabet = self.alphabet.clone();
    let mut ret: Vec<usize> = Vec::new();

    for s in split2 {
      let buffer = format!("{}{}{}", lottery, self.salt.0, alphabet);

      let alpha_len = alphabet.len();
      alphabet = hashids_shuffle(alphabet, &HashidSalt::from(&buffer[0..alpha_len]))?;
      ret.push(unhash(s.to_string(), &alphabet));
    };

    let check_hash = self.encode_vec(&ret);
    if check_hash != hash {
      return Err(Error::InvalidHash)
    };

    Ok(ret)
  }

}


/// This trait is used to group and tag acceptable integer input: u32, u64, i32, i64.
///
/// The algorithm doesn't allow negative integers and floats, 
/// however i32 and i64 are still acccpeted and errors if negative, because Diesel returns i64 integers, 
/// even though I've never seen a database return an negative ID.
/// Converts to usize internally.
pub trait PositiveInteger {
  fn to_usize(self) -> Result<usize, Error>;
}

impl PositiveInteger for u32 {
  fn to_usize(self) -> Result<usize, Error> { Ok(self as usize) }
}

impl PositiveInteger for u64 {
  fn to_usize(self) -> Result<usize, Error> { 
    if self >= std::i64::MAX as u64  {
      return Err(Error::InvalidInputId)
    }
    Ok(self as usize) }
}

impl PositiveInteger for i32 {
  fn to_usize(self) -> Result<usize, Error> {
    if self <= 0  {
      Err(Error::InvalidInputId) 
    } else {
      Ok(self as usize) 
    }
  }
}

impl PositiveInteger for i64 {
  fn to_usize(self) -> Result<usize, Error> {
    if self <= 0  {
      return Err(Error::InvalidInputId)
    }
    // else if self >= std::i64::MAX  {
    //   return Err(Error::InvalidInputId)
    // }
    else {
      Ok(self as usize) 
    }
  }
}


/**
  Following are functions that do not actually use self, so do not belong scoped inside objects.
  They are not public, so API change is fine. Seperating them also greatly facilitates unit testing.
*/

/// Filters separqtors out of the alphabet, and alphabet out of separators
fn get_non_duplicated_string(separators: String, alphabet: String) -> (String, String) {
  let check_separator: HashSet<char> = separators.chars().collect();
  let check_alphabet: HashSet<char> = alphabet.chars().collect();

  let mut modified_separators = String::new();
  let mut modified_alphabet = String::new();
  
  for c in separators.chars() {
    if check_alphabet.contains(&c) {
      modified_separators.push(c);
    }
  };
  
  for c in alphabet.chars() {
    if !check_separator.contains(&c) {
      modified_alphabet.push(c);
    }
  };

  (modified_separators, modified_alphabet)
}


fn get_unique_alphabet(alphabet: String) -> String {
  let mut unique_alphabet: String = String::new();
  let mut check_map = HashSet::new();
  
  for c in alphabet.chars() {
    // insert into a hashset gives a bool, true if it was actually inserted, false if it was already there.
    if check_map.insert(c.clone()) {
      // the result is then used to create the alphabet
      unique_alphabet.push(c);
    }
  };
  unique_alphabet
}

// Function used in both the HashidCode and the builder. 
fn hashids_shuffle(alphabet: String, salt: &HashidSalt) -> Result<String, Error> {
    
  let salt_len = salt.0.len();
  if salt_len <= 0 {
    return Err(Error::MissingSalt)
  };
  if alphabet.len() <= 0 {
    return Err(Error::InvalidAlphabetLength)
  }

  let salt_arr: Vec<char> = salt.0.chars().collect();
  let len = alphabet.len();
  let mut i: usize = len - 1;
  let mut v: usize = 0;
  let mut p: usize = 0;

  let shuffle = &mut alphabet.into_bytes();
  while i > 0 {
    v %= salt_len;
    let t = salt_arr[v] as usize;
    p += t;
    let j = (t + v + p) % i;

    shuffle.swap(i,j);

    i -= 1;
    v += 1; 
  }

  // convert the shuffle [u8] back to String and return that
  let res : String = shuffle.iter().map(|i| *i as char).collect();
  Ok(res)
}

fn unhash(input: String, alphabet: &String) -> usize {
  let mut number= 0;
  let input_slice = input.as_bytes();
  let alpha_slice = alphabet.as_bytes();
  let len = input.len() -1;
  let alpha_len = alphabet.len();

  for (i, v) in input_slice.iter().enumerate() {
    let position = alpha_slice.iter().position(|x| x == v).unwrap_or(0);
    let pow_size = len - i;
    number += position * alpha_len.pow(pow_size as u32);
  };

  number
}

fn hash(mut input: usize, alphabet: &str) -> String {
  let mut hash = "".to_string();
  let len = alphabet.len();

  let mut idx = input % len;
  loop {
    hash = format!("{}{}", alphabet[idx..idx+1].to_string(), hash);
    input /= len;
    if input <= 0 {
      break;
    }
    idx = input % len;
  };
  hash
}

/// converts a HEX String to a vector of integers;
fn hex_to_vec(hex: String) -> Result<Vec<usize>, Error> {
  // check the string is valid HEX
  let _ = i64::from_str_radix(&hex, 16).map_err(|_| Error::NonHexString)?;

  let mut numbers = Vec::new();
  // iterate chars by group of 12, guard div
  let regex = Regex::new(r"[\w\W]{1,12}").unwrap();
  for matcher in regex.find_iter(&hex) {
    let num = format!("1{}", matcher.as_str());
    let v = usize::from_str_radix(&num.to_string(), 16).map_err(|_| Error::NonHexString)?;
    numbers.push(v);
  }
  
  Ok(numbers)
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn decode_hex_string() {
    let hex = "123456789abcdef".to_string();
    let result = hex_to_vec(hex);
    assert_eq!(result, Ok(vec![301490975054524, 7663]));
  }

  #[test]
  fn decode_non_hex_string_error() {
    let data = "4g".to_string();
    let result = hex_to_vec(data);
    assert_eq!(result, Err(Error::NonHexString));
  }

  #[test]
  fn valid_hash() {
    let mut data = 500;
    let result = hash(data, &DEFAULT_ALPHABET.to_string());
    assert_eq!(result, "ie");
    data = 12546843121;
    let result = hash(data, &DEFAULT_ALPHABET.to_string());
    assert_eq!(result, "nRhrdB");
  }

  #[test]
  fn invalid_hash() {
    let data = 0;
    let result = hash(data, &DEFAULT_ALPHABET.to_string());
    assert_eq!(result, "a");
  }

  #[test]
  fn hash_shuffle() {
    let shuffled = hashids_shuffle("anything really goes".to_string(), &HashidSalt::from("this is my salt"));
    assert_eq!(shuffled, Ok(" eagnrlityas oelygnh".to_string()));

  }
}