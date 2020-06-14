extern crate hashids;

use hashids::{HashidBuilder, HashidSalt, Error};

#[test]
fn single_usize_from_single_salt() {
  let ids = HashidBuilder::new().with_hashid_salt(HashidSalt::from("this is my salt")).ok().unwrap();

  let numbers = 12345i64;
  let encode = ids.encode(numbers).unwrap();
  assert_eq!(encode, "NkK9");
  let longs = ids.decode(encode).unwrap();

  assert_eq!(longs, vec![12345]);
}

#[test]
fn decoding_from_different_salt_gives_error() {
  let ids = HashidBuilder::new().with_string_salt("this is my salt".to_string()).ok().unwrap();

  let numbers = 12345;
  let encode = ids.encode(numbers).unwrap();
  assert_eq!(encode, "NkK9");
  
  let ids2 = HashidBuilder::new().with_salt("this is my pepper").ok().unwrap();
  
  let longs = ids2.decode(encode);
  
  assert_eq!(longs, Err(Error::InvalidHash));
}

// #[test]
// fn multiple_integers_to_single_hash() {
//   // I don't know what this could even be used for. But my lack of understanding should not remove a feature.
//   let ids = HashidBuilder::new().with_salt("this is my salt").ok().unwrap();
  
//   let numbers: Vec<i64> = vec![683, 94108, 123, 5];
//   let encode = ids.encode(&numbers).unwrap();
  
//   assert_eq!(encode, "aBMswoO2UB3Sj");
// }

#[test]
fn negative_integers_errors() {
  let codec = HashidBuilder::new().with_salt("this is my salt").ok().unwrap();

  let numbers = -94108;
  let encode = codec.encode(numbers);
  assert_eq!(encode, Err(Error::InvalidInputId));

}

#[test]
fn with_custom_length() {
  let ids = HashidBuilder::new()
                          .with_salt("this is my salt")
                          .with_length(8)
                          .ok().unwrap();
  let numbers= 1;
  let encode = ids.encode(numbers).unwrap();

  assert_eq!(encode, "gB0NV05e");
  assert_eq!(encode.len(), 8);
}

#[test]
fn with_custom_alphabet() {
  let ids = HashidBuilder::new()
                        .with_salt("this is my salt")
                        .with_alphabet("123456789aberzxvtcfhuist".to_string())
                        .ok().unwrap();
  
  let numbers = 1234567;
  let encode = ids.encode(numbers).unwrap();
  
  assert_eq!(encode, "xez268x");
}

// #[test]
// fn with_nonascii_alphabet() {
//   let ids = HashidBuilder::new()
//                         .with_salt("漢字注入注意")
//                         .with_alphabet("あいうえおかきくけこたちつてとさしすせそ".to_string())
//                         .ok();
                        
//   assert_eq!(ids, Err(Error::NonAsciiAlphabet));
//   // let numbers = 1234567;
//   // let encode = ids.encode(numbers).unwrap();
  
// }

#[test]
fn invalid_alphabet_fails() {
  let builder = HashidBuilder::new().with_salt("this is my salt")
          .with_alphabet("abcdefghijklm".to_string())
          .ok();

  match builder {
    Ok(_v) => panic!("Invalid alphabet was accepted"),
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
      let numbers = 12345;
      let encode = ids.encode(numbers).unwrap();
      assert_eq!(encode, "PWbG");
      let longs = ids.decode(encode.clone()).unwrap();
    
      assert_eq!(longs, vec![12345]);
    },

    Err(err) => panic!("Failed building the encoder from environnent variable. Error: {:?}. A test failure might be due to envvar thread unsafety in Unix, try again in isolation.", err)
  }
}

// #[test]
// fn same_integers() {
//   let ids = HashidBuilder::new().with_salt("this is my salt").ok().unwrap();

//   let numbers: Vec<i64> = vec![5, 5, 5, 5];
//   let encode = ids.encode(&numbers).unwrap();

//   assert_eq!(encode, "1Wc8cwcE");
// }

// #[test]
// fn encode_int_series() {
//   let ids = HashidBuilder::new().with_salt("this is my salt").ok().unwrap();

//   let numbers: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
//   let encode = ids.encode(&numbers).unwrap();

//   assert_eq!(encode, "kRHnurhptKcjIDTWC3sx");
// }

#[test]
fn encode_successive_ints() {
  let ids = HashidBuilder::new()
      .with_salt("this is my salt")
      .with_length(2)
      .ok()
      .unwrap();

  let numbers_1 = 1;
  let encode_1 = ids.encode(numbers_1).unwrap();
  let numbers_2 = 2;
  let encode_2 = ids.encode(numbers_2).unwrap();
  let numbers_3 = 3;
  let encode_3 = ids.encode(numbers_3).unwrap();
  let numbers_4 = 4;
  let encode_4 = ids.encode(numbers_4).unwrap();
  let numbers_5 = 5;
  let encode_5 = ids.encode(numbers_5).unwrap();
  let encode_1again = ids.encode(numbers_1).unwrap();

  assert_eq!(encode_1, "NV");
  assert_eq!(encode_2, "6m");
  assert_eq!(encode_3, "yD");
  assert_eq!(encode_4, "2l");
  assert_eq!(encode_5, "rD");
  assert_eq!(encode_1again, "NV");
}

#[test]
fn decode_successive_ints() {
  let ids = HashidBuilder::new().with_salt("this is my salt").ok().unwrap();

  let numbers_1 = 1;
  let encode_1 = ids.encode(numbers_1).unwrap();
  let numbers_2 = 2;
  let encode_2 = ids.encode(numbers_2).unwrap();


  let decoded_1 = ids.decode(encode_1).unwrap();
  assert_eq!(decoded_1, vec![1]);
  let decoded_2 = ids.decode(encode_2).unwrap();
  assert_eq!(decoded_2, vec![2]);
}

#[test]
fn decode_string_out_of_alphabet() {
  let ids = HashidBuilder::new().with_salt("this is my salt").with_alphabet("ABCDEFGHIJKabcdefghijk".to_string()).ok().unwrap();

  let numbers_1 = 1;
  let encode_1 = ids.encode(numbers_1).unwrap();
  assert_eq!(encode_1, "dDKk");
  let decoded_string = "dDzK".to_string();
  let decoded_1 = ids.decode(decoded_string);
  assert_eq!(decoded_1, Err(Error::InvalidHash));
}
