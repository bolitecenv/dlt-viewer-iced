// src/components/dlt_parser.rs
// Parser adapter to convert incoming TCP bytes into DLT messages using dlt-protocol crate

use byteorder::{BigEndian, ReadBytesExt};
use dlt_protocol::*;
use std::io::{Cursor, Read};

#[derive(Debug, Clone)]
pub struct ParsedDltMessage {
    pub standard_header: DltStandardHeader,
    pub standard_header_extra: DltStandardHeaderExtra,
    pub extended_header: DltExtendedHeader,
    pub header_size: usize,
    pub payload: Vec<u8>,
    pub raw_bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum DltParseError {
    InsufficientData,
    InvalidHeader,
    IoError,
}

impl ParsedDltMessage {
    /// Get timestamp as string
    pub fn get_timestamp_string(&self) -> String {
        format!("{}", self.standard_header_extra.tmsp)
    }
    
    /// Get ECU ID as string
    pub fn get_ecu_id(&self) -> String {
        String::from_utf8_lossy(&self.standard_header_extra.ecu)
            .trim_end_matches('\0')
            .to_string()
    }
    
    /// Get Application ID as string
    pub fn get_app_id(&self) -> String {
        String::from_utf8_lossy(&self.extended_header.apid)
            .trim_end_matches('\0')
            .to_string()
    }
    
    /// Get Context ID as string
    pub fn get_context_id(&self) -> String {
        String::from_utf8_lossy(&self.extended_header.ctid)
            .trim_end_matches('\0')
            .to_string()
    }
    
    /// Get message counter
    pub fn get_message_counter(&self) -> u8 {
        self.standard_header.mcnt
    }
    
    /// Parse payload as verbose DLT message
    pub fn parse_payload(&self) -> Option<String> {
        // Check if this is a verbose message
        let msin = self.extended_header.msin;
        let verbose = (msin & 0x01) != 0;
        
        if !verbose {
            // Non-verbose mode - just return raw bytes as hex
            return Some(format!("Non-verbose: {:02X?}", self.payload));
        }
        
        // Parse verbose payload
        let mut parser = PayloadParser::new(&self.payload);
        let noar = self.extended_header.noar;
        
        let mut result = String::new();
        for i in 0..noar {
            match parser.read_next() {
                Ok(value) => {
                    if i > 0 {
                        result.push_str(" ");
                    }
                    match value {
                        DltValue::String(s) => result.push_str(s),
                        DltValue::Bool(v) => result.push_str(&format!("{}", v)),
                        DltValue::I8(v) => result.push_str(&format!("{}", v)),
                        DltValue::I16(v) => result.push_str(&format!("{}", v)),
                        DltValue::I32(v) => result.push_str(&format!("{}", v)),
                        DltValue::I64(v) => result.push_str(&format!("{}", v)),
                        DltValue::U8(v) => result.push_str(&format!("{}", v)),
                        DltValue::U16(v) => result.push_str(&format!("{}", v)),
                        DltValue::U32(v) => result.push_str(&format!("{}", v)),
                        DltValue::U64(v) => result.push_str(&format!("{}", v)),
                        DltValue::U128(v) => result.push_str(&format!("{}", v)),
                        DltValue::F32(v) => result.push_str(&format!("{:.2}", v)),
                        DltValue::F64(v) => result.push_str(&format!("{:.6}", v)),
                        DltValue::Raw(bytes) => result.push_str(&format!("{:02X?}", bytes)),
                    }
                }
                Err(_) => break,
            }
        }
        
        Some(result)
    }
}

/// Parse DLT message from byte slice
/// Returns (ParsedDltMessage, remaining_bytes) or error
pub fn parse_dlt_message(data: &[u8]) -> Result<(ParsedDltMessage, &[u8]), DltParseError> {
    // Check minimum length for standard header
    if data.len() < DLT_STANDARD_HEADER_SIZE {
        return Err(DltParseError::InsufficientData);
    }
    
    let mut cursor = Cursor::new(data);
    
    // Parse standard header
    let htyp = cursor.read_u8().map_err(|_| DltParseError::IoError)?;
    let mcnt = cursor.read_u8().map_err(|_| DltParseError::IoError)?;
    let len = cursor.read_u16::<BigEndian>().map_err(|_| DltParseError::IoError)?;
    
    let standard_header = DltStandardHeader { htyp, mcnt, len };
    
    // Check if we have the full message
    if data.len() < len as usize {
        return Err(DltParseError::InsufficientData);
    }
    
    // Parse header type flags
    let has_ecu_id = (htyp & WEID_MASK) != 0;
    let has_session_id = (htyp & WSID_MASK) != 0;
    let has_timestamp = (htyp & WTMS_MASK) != 0;
    let has_extended_header = (htyp & UEH_MASK) != 0;
    
    // Parse standard header extra
    let mut ecu = [0u8; DLT_ID_SIZE];
    let mut seid = 0u32;
    let mut tmsp = 0u32;
    
    if has_ecu_id {
        cursor.read_exact(&mut ecu).map_err(|_| DltParseError::IoError)?;
    }
    
    if has_session_id {
        seid = cursor.read_u32::<BigEndian>().map_err(|_| DltParseError::IoError)?;
    }
    
    if has_timestamp {
        tmsp = cursor.read_u32::<BigEndian>().map_err(|_| DltParseError::IoError)?;
    }
    
    let standard_header_extra = DltStandardHeaderExtra { ecu, seid, tmsp };
    
    // Parse extended header
    let mut extended_header = DltExtendedHeader::default();
    if has_extended_header {
        let msin = cursor.read_u8().map_err(|_| DltParseError::IoError)?;
        let noar = cursor.read_u8().map_err(|_| DltParseError::IoError)?;
        let mut apid = [0u8; DLT_ID_SIZE];
        let mut ctid = [0u8; DLT_ID_SIZE];
        cursor.read_exact(&mut apid).map_err(|_| DltParseError::IoError)?;
        cursor.read_exact(&mut ctid).map_err(|_| DltParseError::IoError)?;
        
        extended_header = DltExtendedHeader { msin, noar, apid, ctid };
    }
    
    // Calculate header size
    let current_pos = cursor.position() as usize;
    let header_size = current_pos;
    
    // Extract payload
    let payload_size = len as usize - current_pos;
    let mut payload = vec![0u8; payload_size];
    cursor.read_exact(&mut payload).map_err(|_| DltParseError::IoError)?;
    
    // Get raw bytes of this message
    let raw_bytes = data[..len as usize].to_vec();
    
    let parsed_message = ParsedDltMessage {
        standard_header,
        standard_header_extra,
        extended_header,
        header_size,
        payload,
        raw_bytes,
    };
    
    // Return parsed message and remaining bytes
    let remaining = &data[len as usize..];
    Ok((parsed_message, remaining))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_message() {
        // Simple DLT message with all headers
        let data: [u8; 32] = [
            0x35, 0x00, 0x00, 0x20, // Standard header (htyp=0x35, mcnt=0, len=32)
            0x45, 0x43, 0x55, 0x31, // ECU ID "ECU1"
            0x00, 0x00, 0x00, 0x01, // Session ID = 1
            0x12, 0x34, 0x56, 0x78, // Timestamp
            0x41, 0x01,             // Extended header (msin=0x41, noar=1)
            0x44, 0x41, 0x31, 0x00, // App ID "DA1\0"
            0x44, 0x43, 0x31, 0x00, // Context ID "DC1\0"
            0x00, 0x00, 0x00, 0x00, // Payload (4 bytes)
        ];
        
        let result = parse_dlt_message(&data);
        assert!(result.is_ok());
        
        let (msg, remaining) = result.unwrap();
        assert_eq!(msg.standard_header.len, 32);
        assert_eq!(msg.get_ecu_id(), "ECU1");
        assert_eq!(remaining.len(), 0);
    }
}
