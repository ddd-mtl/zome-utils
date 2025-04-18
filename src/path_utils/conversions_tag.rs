//use std::convert::Infallible;
use hdk::prelude::*;
use hdk::prelude::holo_hash::{HashType, holo_hash_decode_unchecked, holo_hash_encode};


/// Convert String to LinkTag
pub fn str2tag(tag_str: &str) -> LinkTag {
  return LinkTag::new(tag_str.as_bytes().to_vec());
}


/// Convert LinkTag to String
pub fn tag2str(tag: &LinkTag) -> ExternResult<String> {
  let vec = &tag.0;
  let Ok(str) = std::str::from_utf8(vec)
    else { return Err(wasm_error!(WasmErrorInner::Guest("Failed to parse utf8 string from link tag".to_string()))) };
  Ok(str.to_string())
}



///
pub fn hash2tag<T: HashType>(hash: HoloHash<T>) -> LinkTag {
  let str = holo_hash_encode(hash.get_raw_39());
  return str2tag(&str);
}


///
pub fn tag2hash<T: HashType>(tag: &LinkTag) -> ExternResult<HoloHash<T>> {
  let hash_str = tag2str(&tag)?;
  // if hash_str == "" {
  //
  // }
  let raw_hash = holo_hash_decode_unchecked(&hash_str)
     .map_err(|e|wasm_error!(SerializedBytesError::Deserialize(e.to_string())))?;
  let hash = HoloHash::<T>::try_from_raw_39(raw_hash)
     .map_err(|e|wasm_error!(SerializedBytesError::Deserialize(e.to_string())))?;
  Ok(hash)
}


/// Convert the i64 timestamp stored in the tag as Vec<u8>
pub fn tag2Ts(tag: LinkTag) -> Timestamp {
  let bytes: [u8;8] = tag.0.try_into().unwrap();
  let ts = i64::from_le_bytes(bytes);
  return Timestamp::from_micros(ts);
}


///
pub fn ts2Tag(ts: Timestamp) -> LinkTag {
  LinkTag::new(ts.0.to_le_bytes().to_owned())
}


///
pub fn obj2Tag<T: serde::Serialize + std::fmt::Debug + Clone>(obj: T) -> ExternResult<LinkTag> {
  let data = encode(&obj)
     .map_err(|e|wasm_error!(SerializedBytesError::Serialize(e.to_string())))?;
  Ok(LinkTag::new(data))
}


// ///
// pub fn tag2Obj<'a, T: Deserialize<'a> + std::fmt::Debug + Clone>(tag: LinkTag) -> ExternResult<T> {
//    let data: T = decode(&tag.into_inner())
//      .map_err(|e| wasm_error!(SerializedBytesError::Deserialize(e.to_string())))?;
//   Ok(data)
// }


// ///
// pub fn obj2Tag<T: Into<hdk::prelude::SerializedBytes>>(obj: T) -> ExternResult<LinkTag> {
//   let sb = SerializedBytes::try_from(obj)?;
//   Ok(LinkTag::new(sb.bytes().to_owned()))
// }
//
//
// ///
// pub fn tag2Obj<T: From<hdk::prelude::SerializedBytes>>(tag: LinkTag) -> ExternResult<T> {
//   let res = SerializedBytes::from(UnsafeBytes::from(tag.0)).try_into()
//                                                            .map_err(|e: Infallible| wasm_error!(SerializedBytesError::Deserialize(e.to_string())))?;
//   Ok(res)
// }

