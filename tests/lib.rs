extern crate hashids;

use hashids::{HashIdBuilder, HashidSalt, HashIdBuilderError};

#[test]
fn single_usize_from_single_salt() {
  let ids = match HashIdBuilder::new_with_salt(HashidSalt::from("this is my salt")){
    Ok(v) => { v }
    Err(e) => {
      panic!("Couldn't create ids. Error: {:?}", e);
    }
  };

  let numbers: Vec<i64> = vec![12345];
  let encode = ids.encode(&numbers);
  assert_eq!(encode, "NkK9");
  let longs = ids.decode(encode.clone());

  assert_eq!(longs, vec![12345]);
}

#[test]
fn decoding_from_different_salt_gives_empty_vec() {
  // I don't agree with this API design. It should be an error, not an null
  let ids = match HashIdBuilder::new_with_salt(HashidSalt::from("this is my salt")) {
    Ok(v) => { v }
    Err(e) => {
      panic!("Couldn't create ids. Error: {:?}", e);
    }
  };

  let numbers: Vec<i64> = vec![12345];
  let encode = ids.encode(&numbers);
  assert_eq!(encode, "NkK9");
  
  let ids2 = match HashIdBuilder::new_with_salt(HashidSalt::from("this is my pepper")) {
    Ok(v) => { v }
    Err(e) => {
      panic!("Couldn't create ids2. Error: {:?}", e);
    }
  };
  
  let longs = ids2.decode(encode);
  
  assert_eq!(longs, vec![]);
}

#[test]
fn multiple_integers_to_single_hash() {
  // I don't know what this could even be used for. But my lack of understanding should not remove a feature.
  let ids = match HashIdBuilder::new_with_salt(HashidSalt::from("this is my salt")) {
    Ok(v) => { v }
    Err(e) => {
      panic!("Couldn't create ids. Error: {:?}", e);
    }
  };
  
  let numbers: Vec<i64> = vec![683, 94108, 123, 5];
  let encode = ids.encode(&numbers);
  
  assert_eq!(encode, "aBMswoO2UB3Sj");
}

#[test]
#[should_panic]
fn negative_integers_panics() {
  // This should be made into a proper error. Poor API design again.
  let ids = match HashIdBuilder::new_with_salt(HashidSalt::from("this is my salt")) {
    Ok(v) => { v }
    Err(e) => {
      panic!("Couldn't create ids. Error: {:?}", e);
    }
  };

  let numbers: Vec<i64> = vec![683, -94108, 123, 5];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "aBMswoO2UB3Sj");
}

#[test]
fn with_custom_length() {
  let ids = match HashIdBuilder::new_with_salt_and_min_length(HashidSalt::from("this is my salt"), 8) {
    Ok(v) => { v }
    Err(e) => {
      panic!("Couldn't create ids. Error: {:?}", e);
    }
  };

  let numbers: Vec<i64> = vec![1];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "gB0NV05e");
}

#[test]
fn raw_new() {
  let ids = match HashIdBuilder::new(HashidSalt::from("this is my salt"), 0,  "0123456789abcdef".to_string()) {
    Ok(v) => { v }
    Err(e) => {
      panic!("Couldn't create ids. Error: {:?}", e)
    }
  };
  
  let numbers: Vec<i64> = vec![1234567];
  let encode = ids.encode(&numbers);
  
  assert_eq!(encode, "b332db5");
}


#[test]
fn invalid_alphabet_fails() {
  // The alphabet is invalid because it is under 16 chars long (as defined by const MIN_ALPHABET_LENGTH)
  match HashIdBuilder::new(HashidSalt::from("this is my salt"), 0,  "abcdefghijklm".to_string()) {
    Ok(v) => panic!("Invalid alphabet was accepted {}", v.alphabet),
    Err(e) => assert_eq!(e, HashIdBuilderError::InvalidAlphabetLength)
  }
}

#[test]
fn same_integers() {
  let ids_some = HashIdBuilder::new_with_salt(HashidSalt::from("this is my salt"));
  let ids = match ids_some {
    Ok(v) => { v }
    Err(e) => {
      panic!("Couldn't create ids. Error: {:?}", e);
    }
  };

  let numbers: Vec<i64> = vec![5, 5, 5, 5];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "1Wc8cwcE");
}

#[test]
fn encode_int_series() {
  let ids = match HashIdBuilder::new_with_salt(HashidSalt::from("this is my salt")) {
    Ok(v) => { v }
    Err(e) => {
      panic!("Couldn't create ids. Error: {:?}", e);
    }
  };

  let numbers: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "kRHnurhptKcjIDTWC3sx");
}

#[test]
fn encode_successive_ints() {
  let ids = match  HashIdBuilder::new_with_salt(HashidSalt::from("this is my salt")) {
    Ok(v) => { v }
    Err(e) => {
      panic!("Couldn't create ids. Error: {:?}", e);
    }
  };

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
  let ids = match  HashIdBuilder::new_with_salt(HashidSalt::from("this is my salt")) {
    Ok(v) => { v }
    Err(e) => {
      panic!("Couldn't create ids. Error: {:?}", e);
    }
  };

  let numbers_1: Vec<i64> = vec![1];
  let encode_1 = ids.encode(&numbers_1);
  let numbers_2: Vec<i64> = vec![2];
  let encode_2 = ids.encode(&numbers_2);


  let decoded_1 = ids.decode(encode_1);
  assert_eq!(decoded_1, vec![1]);
  let decoded_2 = ids.decode(encode_2);
  assert_eq!(decoded_2, vec![2]);
}
