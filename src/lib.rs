/// Create a hashid
/// 
/// Use the builder to configure, then use the codec to encode and decode.


use std::collections::{HashSet};
use regex::Regex;

const ENV_KEY: &'static str =  "HASHID_SALT";
const DEFAULT_ALPHABET: &'static str =  "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
const DEFAULT_MIN_LENGTH : usize = 4;
const DEFAULT_SEPARATORS: &'static str = "cfhistuCFHISTU";
const SEPARATOR_DIV: f32 = 3.5;
const GUARD_DIV: u32 = 12;
const MIN_ALPHABET_LENGTH: usize = 16;

#[derive(Debug, PartialEq)]
pub enum Error { 
  MissingSalt,
  InvalidAlphabetLength,
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

  /// Use this builder to setup the hashid encoder/decoder [hashid::HashidCodec].
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
  ///                                    .with_salt("my salt")
  ///                                    .with_alphabet("12345789abcedef~!@#$%^&*()_+".to_string())
  ///                                    .with_length(16)
  ///                                    .ok();
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
  pub fn with_alphabet(mut self, alphabet: String) -> HashidBuilder {
    self.alphabet = Some(alphabet);
    self
  }

  // length-related methods
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
    let alphabet = if let Some(custom) = self.alphabet { custom } else { DEFAULT_ALPHABET.to_string() };
    let unique_alphabet = HashidBuilder::get_unique_alphabet(alphabet);
    if unique_alphabet.len() < MIN_ALPHABET_LENGTH {
      return Err(Error::InvalidAlphabetLength);
    };
    // get custom salt, set from builder function or by environnment
    let salt = if let Some(custom) = self.salt { custom } else { 
      let by_env = std::env::var(ENV_KEY);
      match by_env {
        Ok(var) => HashidSalt::from(var),
        Err(_) => return Err(Error::MissingSalt)
      }
    };
    
    let min_length = if let Some(custom) = self.min_length { custom } else { DEFAULT_MIN_LENGTH };
    
    let (t_separators, mut t_alphabet) = HashidBuilder::get_non_duplicated_string(DEFAULT_SEPARATORS.to_string(), unique_alphabet);
    let mut shuffled_separators = hashids_shuffle(t_separators.clone(), &salt);
    let alphabet_len = t_alphabet.len();
    
    let shuffled_separators_len = shuffled_separators.len();

    if shuffled_separators_len <= 0 || ((alphabet_len/shuffled_separators_len) as f32) > SEPARATOR_DIV {
      let mut seps_len =  ((alphabet_len as f32) / SEPARATOR_DIV) as usize;
      if seps_len == 1 {
        seps_len = 2;
      }

      if seps_len > shuffled_separators_len {
        let diff = seps_len - shuffled_separators_len;

        shuffled_separators.push_str(&t_alphabet[..diff]);
        t_alphabet = t_alphabet[diff..].to_string();
      } else {
        shuffled_separators = shuffled_separators[..seps_len].to_string();
      }
    }

    let mut shuffled_alphabet = hashids_shuffle(t_alphabet, &salt);

    let guard_count = (alphabet_len as f32 / GUARD_DIV as f32).ceil() as usize;

    let t_guards;

    if alphabet_len < 3 {
      t_guards = shuffled_separators[..guard_count].to_string();
      shuffled_separators = shuffled_separators[guard_count..].to_string();
    } else {
      t_guards = shuffled_alphabet[..guard_count].to_string();
      shuffled_alphabet = shuffled_alphabet[guard_count..].to_string();
    }

    Ok(HashidCodec {
      salt: salt,
      min_hash_length: min_length,
      guards: t_guards,
      separators: shuffled_separators,
      alphabet: shuffled_alphabet
    })
  }

  fn get_non_duplicated_string(separators: String, alphabet: String) -> (String, String) {
    let check_separator = string_to_set(&separators);
    let check_alphabet = string_to_set(&alphabet);

    let mut modified_separators = String::new();
    let mut modified_alphabet = String::new();
    
    for c in separators.chars() {
      if check_alphabet.contains(&c) {
        modified_separators.push(c);
      }
    }
    
    for c in alphabet.chars() {
      if !check_separator.contains(&c) {
        modified_alphabet.push(c);
      }
    }
    
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
    }
    unique_alphabet
  }
}



/// This struct holds the processing. It can only be created from a HashidBuilder::new().ok();
pub struct HashidCodec {
  salt: HashidSalt,
  pub alphabet: String,
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
  pub fn encode_hex(&self, hex: String) -> String {
    let regex1 = Regex::new(r"^[0-9a-fA-F]+$").unwrap();
    if regex1.is_match(&hex.to_string()) == false {
      return String::new();
    }

    let mut numbers: Vec<i64> = Vec::new();
    let regex2 = Regex::new(r"[\w\W]{1,12}").unwrap();
    for matcher in regex2.find_iter(&hex.to_string()) {
      let mut num = String::new();
      num.push('1');
      num.push_str(&hex[matcher.range()]);
      let v: i64 = i64::from_str_radix(&num.to_string(), 16).unwrap();
      numbers.push(v);
    }
    
    self.encode(&numbers)
  }

  pub fn decode_hex(&self, hash: String) -> String {
    let mut ret = String::new();
    let numbers = self.decode(hash);
    for number in numbers.iter() {
      let r = format!("{:x}", number);
      ret.push_str(&r[1..]);
    }

    ret
  }

  pub fn encode(&self, numbers: &Vec<i64>) -> String {
    if numbers.len() == 0 {
      return "".to_string();
    }

    for number in numbers.iter() {
      if *number > 9007199254740992 {
        return "".to_string();
      }
    }

    self._encode(numbers)
  }

  pub fn decode(&self, hash: String) -> Vec<i64> {
    let ret : Vec<i64> = Vec::new();
    if hash.len() == 0 {
      return ret;
    }

    self._decode(hash)
  }

  fn _decode(&self, hash: String) -> Vec<i64> {
    let mut regexp = String::new();
    regexp.push('[');
    regexp.push_str(&self.guards[..]);
    regexp.push(']');

    let re = Regex::new(&regexp[..]).unwrap();
    let t_hash = re.replace_all(&hash[..], " ");

    let split1: Vec<&str> = t_hash[..].split_whitespace().collect();
    let mut i = 0;

    let len = split1.len();
    if len == 3 || len == 2 {
      i = 1;
    }
    let mut hash_breakdown = split1[i].to_string();

    let lottery = hash_breakdown[0..1].to_string();
    hash_breakdown = hash_breakdown[1..].to_string();

    let mut regexp2 = String::new();
    regexp2.push('[');
    regexp2.push_str(&self.separators[..]);
    regexp2.push(']');

    let re2 = Regex::new(&regexp2[..]).unwrap();
    hash_breakdown = re2.replace_all(&hash_breakdown, " ").to_string();

    let split2: Vec<&str> = hash_breakdown[..].split_whitespace().collect();

    let mut alphabet = self.alphabet.clone();

    let mut ret: Vec<i64> = Vec::new();

    for s in split2 {
      let sub_hash = s.to_string();
      let mut buffer = String::new();
      buffer.push_str(&lottery[..]);
      buffer.push_str(&self.salt.0[..]);
      buffer.push_str(&alphabet.clone()[..]);

      let alpha_len = alphabet.len();
      alphabet = hashids_shuffle(alphabet, &HashidSalt::from(&buffer[0..alpha_len]));
      ret.push(HashidCodec::unhash(sub_hash, alphabet.clone()));
    }

    let check_hash = self._encode(&ret);
    if check_hash != hash {
      return Vec::new();
    }

    ret
  }

  fn index_of(input :&[u8], v: u8) -> i64 {
    let mut i = 0;
    for s in input.iter() {
      if *s == v {
        return i;
      }

      i += 1;
    }

    return -1;
  }

  fn unhash(input: String, alphabet: String) -> i64 {
    let mut number: i64 = 0;
    let input_slice = input.as_bytes();
    let alpha_slice = alphabet.as_bytes();
    let len = input.len();
    let alpha_len = alphabet.len() as i64;
    let mut i: usize = 0;
    loop {
      if i >= len {
        break;
      }

      let v = input_slice[i] as usize;
      let pos = HashidCodec::index_of(alpha_slice, v as u8);
      let pow_size = (len - i - 1) as u32;
      number += (pos * alpha_len.pow(pow_size)) as i64;
      i += 1;
    }

    number
  }

  fn hash(input: i64, alphabet: String) -> String {
    let mut t_in = input;
    let mut hash = "".to_string();
    let len = alphabet.len() as i64;

    loop {
      let idx = (t_in % len) as usize;
      let mut t = alphabet[idx..idx+1].to_string();
      t.push_str(&hash[..]);
      hash = t;
      t_in /= len;

      if t_in <= 0 {
        break;
      }
    }

    hash
  }

  fn _encode(&self, numbers: &Vec<i64>) -> String {
    let mut number_hash_int  = 0;
    let mut count = 100;
    for number in numbers.iter() {
      number_hash_int += *number % count;
      count += 1;
    }

    let idx = (number_hash_int % (self.alphabet.len() as i64)) as usize;
    let ret = self.alphabet[idx..idx+1].to_string();
    let mut ret_str = ret.clone();

    let mut t_alphabet = self.alphabet.clone();
    let mut i = 0;
    let len = self.separators.len() as i64;
    let last_len = count - 100;
    for number in numbers.iter() {
      let mut buffer = ret.clone();
      buffer.push_str(&self.salt.0[..]);
      buffer.push_str(&t_alphabet[..]);
      t_alphabet = hashids_shuffle(t_alphabet.clone(), &HashidSalt::from(&buffer[0..t_alphabet.len()]));
      let last = HashidCodec::hash(*number, t_alphabet.clone());

      ret_str.push_str(&last[..]);

      if (i + 1) < last_len {
        let mut v = *number % (last.as_bytes()[0] as i64 + i);
        v = v % len;
        ret_str.push(self.separators.as_bytes()[v as usize] as char);
      }
      i += 1;
    }

    if ret_str.len() < self.min_hash_length {
      let guard_idx = (number_hash_int + ret_str.clone().into_bytes()[0] as i64) as usize % self.guards.len();
      let guard = self.guards[guard_idx..guard_idx+1].to_string();
      let mut t = guard.clone();
      t.push_str(&ret_str[..]);
      ret_str = t;

      if ret_str.len() < self.min_hash_length {
        let guard_idx = (number_hash_int + ret_str.clone().into_bytes()[2] as i64) as usize % self.guards.len();
        ret_str.push_str(&self.guards[guard_idx..guard_idx+1]);
      }
    }

    let half_len = t_alphabet.len() / 2;
    while ret_str.len() < self.min_hash_length {
      t_alphabet = hashids_shuffle(t_alphabet.clone(), &HashidSalt::from(t_alphabet));
      let mut t_ret = "".to_string();
      t_ret.push_str(&t_alphabet[half_len..]);
      t_ret.push_str(&ret_str[..]);
      t_ret.push_str(&t_alphabet[0..half_len]);
      ret_str = t_ret;

      let excess = ret_str.len() as i64 - self.min_hash_length as i64;
      if excess > 0 {
        let start_pos = (excess as i64 / 2) as usize;
        ret_str = ret_str[start_pos..start_pos + self.min_hash_length].to_string();
      }
    }

    ret_str
  }
}


fn hashids_shuffle(alphabet: String, salt: &HashidSalt) -> String {
    
  let salt_len = salt.0.len();
  if salt_len <= 0 {
    return alphabet;
  }

  let arr = salt.0.as_bytes();
  let len = alphabet.len();
  let mut bytes = alphabet.into_bytes();
  let shuffle = &mut bytes[..];

  let mut i: usize = len-1;
  let mut v: usize = 0;
  let mut p: usize = 0;

  while i > 0 {
    v %= salt_len;
    let t = arr[v] as usize;
    p += t;
    let j = (t + v + p) % i;

    shuffle.swap(i,j);

    i=i-1;
    v=v+1; 
  }

  let mut shuffled_alphabet = String::with_capacity(len);
  for i in 0..len {
    shuffled_alphabet.push(shuffle[i] as char);
  }

  shuffled_alphabet
}

fn string_to_set(string: &String) -> HashSet<char> {
  let mut set = HashSet::new();
  for c in string.chars() {
    set.insert(c.clone());
  }
  
  set
}