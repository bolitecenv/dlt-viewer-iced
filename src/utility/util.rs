use bincode::{Decode, Encode, config, error::{DecodeError, EncodeError}};

pub fn serialize_message<T>(msg: &T) -> Result<Vec<u8>, EncodeError> 
where
    T: Encode,
{
    bincode::encode_to_vec(msg, config::standard())
}

pub fn deserialize_message<T>(data: &[u8]) -> Result<T, DecodeError>
where
    T: Decode<()>,
{
    bincode::decode_from_slice(data, config::standard())
        .map(|(result, _)| result)
}