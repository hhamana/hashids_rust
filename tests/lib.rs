extern crate hashids;

use hashids::{HashidBuilder, HashidSalt, Error};

#[test]
fn single_usize_from_single_salt() {
  let ids = HashidBuilder::new().with_hashid_salt(HashidSalt::from("this is my salt")).ok().unwrap();

  let numbers: Vec<i64> = vec![12345];
  let encode = ids.encode(&numbers);
  assert_eq!(encode, "NkK9");
  let longs = ids.decode(encode.clone());

  assert_eq!(longs, vec![12345]);
}

#[test]
fn decoding_from_different_salt_gives_empty_vec() {
  // I don't agree with this API design. It should be an error, not an null
  let ids = HashidBuilder::new().with_string_salt("this is my salt".to_string()).ok().unwrap();

  let numbers: Vec<i64> = vec![12345];
  let encode = ids.encode(&numbers);
  assert_eq!(encode, "NkK9");
  
  let ids2 = HashidBuilder::new().with_salt("this is my pepper").ok().unwrap();
  
  let longs = ids2.decode(encode);
  
  assert_eq!(longs, vec![]);
}

#[test]
fn multiple_integers_to_single_hash() {
  // I don't know what this could even be used for. But my lack of understanding should not remove a feature.
  let ids = HashidBuilder::new().with_salt("this is my salt").ok().unwrap();
  
  let numbers: Vec<i64> = vec![683, 94108, 123, 5];
  let encode = ids.encode(&numbers);
  
  assert_eq!(encode, "aBMswoO2UB3Sj");
}

#[test]
#[should_panic]
fn negative_integers_panics() {
  // This should be made into a proper error. Poor API design again.
  let ids = HashidBuilder::new().with_salt("this is my salt").ok().unwrap();

  let numbers: Vec<i64> = vec![683, -94108, 123, 5];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "aBMswoO2UB3Sj");
}

#[test]
fn with_custom_length() {
  let ids = HashidBuilder::new()
                          .with_salt("this is my salt")
                          .with_length(8)
                          .ok().unwrap();
  let numbers: Vec<i64> = vec![1];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "gB0NV05e");
}

#[test]
fn with_custom_alphabet() {
  let ids = HashidBuilder::new()
                        .with_salt("this is my salt")
                        .with_alphabet("123456789aberzxvtcfhuist".to_string())
                        .ok().unwrap();
  
  let numbers: Vec<i64> = vec![1234567];
  let encode = ids.encode(&numbers);
  
  assert_eq!(encode, "xez268x");
}
#[test]
fn invalid_alphabet_fails() {
  let builder = HashidBuilder::new().with_salt("this is my salt")
          .with_alphabet("abcdefghijklm".to_string())
          .ok();

  match builder {
    Ok(v) => panic!("Invalid alphabet was accepted {}", v.alphabet),
    Err(e) => assert_eq!(e, Error::InvalidAlphabetLength)
  }
}

#[test]
fn without_salt_error() {
  std::env::remove_var("HASHID_SALT");
  match HashidBuilder::new().ok() {
    Ok(_) => panic!("Created a HashidCodec without salt. A test failure might be due to envvar thread unsafety in Unix, try again in isolation."),
    Err(err) => assert_eq!(err, Error::MissingSalt)
  }
}

#[test]
fn with_envvar_salt() {
  std::env::set_var("HASHID_SALT", "organic salt");
  let the_most_simple_builder = HashidBuilder::new().ok();
  match the_most_simple_builder {
    Ok(ids) => {
      let numbers: Vec<i64> = vec![12345];
      let encode = ids.encode(&numbers);
      assert_eq!(encode, "PWbG");
      let longs = ids.decode(encode.clone());
    
      assert_eq!(longs, vec![12345]);
    },

    Err(err) => panic!("Failed building the encoder from environnent variable. Error: {:?}. A test failure might be due to envvar thread unsafety in Unix, try again in isolation.", err)
  }
}

#[test]
fn same_integers() {
  let ids = HashidBuilder::new().with_salt("this is my salt").ok().unwrap();

  let numbers: Vec<i64> = vec![5, 5, 5, 5];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "1Wc8cwcE");
}

#[test]
fn encode_int_series() {
  let ids = HashidBuilder::new().with_salt("this is my salt").ok().unwrap();

  let numbers: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "kRHnurhptKcjIDTWC3sx");
}

#[test]
fn encode_successive_ints() {
  let ids = HashidBuilder::new()
      .with_salt("this is my salt")
      .with_length(2)
      .ok().unwrap();

  let numbers_1: Vec<i64> = vec![1];
  let encode_1 = ids.encode(&numbers_1);
  let numbers_2: Vec<i64> = vec![2];
  let encode_2 = ids.encode(&numbers_2);
  let numbers_3: Vec<i64> = vec![3];
  let encode_3 = ids.encode(&numbers_3);
  let numbers_4: Vec<i64> = vec![4];
  let encode_4 = ids.encode(&numbers_4);
  let numbers_5: Vec<i64> = vec![5];
  let encode_5 = ids.encode(&numbers_5);

  assert_eq!(encode_1, "NV");
  assert_eq!(encode_2, "6m");
  assert_eq!(encode_3, "yD");
  assert_eq!(encode_4, "2l");
  assert_eq!(encode_5, "rD");
}

#[test]
fn decode_successive_ints() {
  let ids = HashidBuilder::new().with_salt("this is my salt").ok().unwrap();

  let numbers_1: Vec<i64> = vec![1];
  let encode_1 = ids.encode(&numbers_1);
  let numbers_2: Vec<i64> = vec![2];
  let encode_2 = ids.encode(&numbers_2);


  let decoded_1 = ids.decode(encode_1);
  assert_eq!(decoded_1, vec![1]);
  let decoded_2 = ids.decode(encode_2);
  assert_eq!(decoded_2, vec![2]);
}
