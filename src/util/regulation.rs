use std::{collections::HashMap, io::Error, str::FromStr, sync::{Mutex, RwLock}};

use aes::cipher::{block_padding::NoPadding, BlockDecryptMut, KeyIvInit};
use binary_reader::{BinaryReader, Endian};
use once_cell::sync::{Lazy, OnceCell};

use crate::{db::{accessory_name::accessory_name::ACCESSORY_NAME, aow_name::aow_name::AOW_NAME, armor_name::armor_name::ARMOR_NAME, item_name::item_name::ITEM_NAME, weapon_name::weapon_name::WEAPON_NAME}, save::save::save::Save, util::{param_structs::{EQUIP_PARAM_ACCESSORY_ST, EQUIP_PARAM_GEM_ST, EQUIP_PARAM_GOODS_ST, EQUIP_PARAM_PROTECTOR_ST, EQUIP_PARAM_WEAPON_ST}, params::params::{Row, PARAM}}};

use super::{bnd4::bnd4::BND4, params::params::Param};

pub static PARAMS: Lazy<RwLock<HashMap<Param, Vec<u8>>>> = Lazy::new(|| RwLock::new(Default::default()));

fn invalid_data(msg: impl Into<String>) -> Error {
    Error::new(std::io::ErrorKind::InvalidData, msg.into())
}

fn expect_eq<T: PartialEq + std::fmt::Debug>(actual: T, expected: T, what: &str) -> Result<(), Error> {
    if actual != expected {
        return Err(invalid_data(format!("{what}: expected {expected:?}, got {actual:?}")));
    }
    Ok(())
}

pub struct Regulation;

impl Regulation {
    pub fn init_params(save: &Save) {
        let res = Regulation::params_from_regulation(&save.save_type.get_regulation());

        
        match res {
            Ok(res) => *PARAMS.write().unwrap() = res,
            Err(err) => println!("{err}"),
        }

        
    }

    pub fn equip_accessory_param_map() -> &'static HashMap<u32, Row<EQUIP_PARAM_ACCESSORY_ST>> {
        static ACCESSORY_PARAM_MAP: OnceCell<HashMap<u32, Row<EQUIP_PARAM_ACCESSORY_ST>>> = OnceCell::new();
        ACCESSORY_PARAM_MAP.get_or_init(|| { 
            let mut map = Self::get_param_map::<EQUIP_PARAM_ACCESSORY_ST>(&Param::EquipParamAccessory);
            Self::try_fill_names::<EQUIP_PARAM_ACCESSORY_ST>(&mut map, &ACCESSORY_NAME);
            map
        })
    }

    pub fn equip_gem_param_map() -> &'static HashMap<u32, Row<EQUIP_PARAM_GEM_ST>> {
        static GEM_PARAM_MAP: OnceCell<HashMap<u32, Row<EQUIP_PARAM_GEM_ST>>> = OnceCell::new();
        GEM_PARAM_MAP.get_or_init(|| { 
            let mut map = Self::get_param_map::<EQUIP_PARAM_GEM_ST>(&Param::EquipParamGem); 
            Self::try_fill_names::<EQUIP_PARAM_GEM_ST>(&mut map, &AOW_NAME);
            map
        })
    }

    pub fn equip_goods_param_map() -> &'static HashMap<u32, Row<EQUIP_PARAM_GOODS_ST>> {
        static GOOD_PARAM_MAP: OnceCell<HashMap<u32, Row<EQUIP_PARAM_GOODS_ST>>> = OnceCell::new();
        GOOD_PARAM_MAP.get_or_init(|| { 
            let mut map = Self::get_param_map::<EQUIP_PARAM_GOODS_ST>(&Param::EquipParamGoods); 
            Self::try_fill_names::<EQUIP_PARAM_GOODS_ST>(&mut map, &ITEM_NAME);
            map
        })
    }

    pub fn equip_protectors_param_map() -> &'static HashMap<u32, Row<EQUIP_PARAM_PROTECTOR_ST>> {
        static PROTECTOR_PARAM_MAP: OnceCell<HashMap<u32, Row<EQUIP_PARAM_PROTECTOR_ST>>> = OnceCell::new();
        PROTECTOR_PARAM_MAP.get_or_init(|| { 
            let mut map = Self::get_param_map::<EQUIP_PARAM_PROTECTOR_ST>(&Param::EquipParamProtector); 
            Self::try_fill_names::<EQUIP_PARAM_PROTECTOR_ST>(&mut map, &ARMOR_NAME);
            map
        })
    }

    pub fn equip_weapon_params_map() -> &'static HashMap<u32, Row<EQUIP_PARAM_WEAPON_ST>> {
        static WEAPON_PARAM_MAP: OnceCell<HashMap<u32, Row<EQUIP_PARAM_WEAPON_ST>>> = OnceCell::new();
        WEAPON_PARAM_MAP.get_or_init(|| { 
            let mut map = Self::get_param_map::<EQUIP_PARAM_WEAPON_ST>(&Param::EquipParamWeapon); 
            Self::try_fill_names::<EQUIP_PARAM_WEAPON_ST>(&mut map, &WEAPON_NAME);
            map
        })
    }

    fn get_param_map<T>(param: &Param) -> HashMap<u32, Row<T>> where T: Default + Clone {
        let params = PARAMS.read().unwrap();
        let bytes = match params.get(param) {
            Some(bytes) => bytes,
            None => return HashMap::new(),
        };
        match PARAM::<T>::from_bytes(bytes) {
            Ok(param) => param.rows.into_iter()
                .map(|row| (row.id, row))
                .collect::<HashMap<u32, Row<T>>>(),
            Err(_) => HashMap::new(),
        }
    }
    
    fn try_fill_names<T>(rows: &mut HashMap<u32, Row<T>>, map: &Lazy<Mutex<HashMap<u32, &str>>>) where T: Default + Clone {
        rows.iter_mut().for_each(|(_, entry)| {
            entry.name = match map.lock().unwrap().get(&(entry.id)) {
                Some(name) => if !name.is_empty() {name.to_string()} else {format!("[UNKOWN_{}]", entry.id)},
                None => format!("[UNKOWN_{}]", entry.id),
            };
        });
    }

    pub fn params_from_regulation(bytes: &[u8]) -> Result<HashMap<Param, Vec<u8>>, Error>{
        let decrypted = Self::decrypt(&bytes)?;
        let decompressed = Self::decompress(&decrypted)?;
        let res = Self::unpack(&decompressed)?;
        let mut params: HashMap<Param, Vec<u8>> = HashMap::new();

        for file in &res.files {
            let name_no_dir = match file.name.split("\\").last() {
                Some(n) => n,
                None => continue,
            };
            let file_name = match name_no_dir.split(".").nth(0) {
                Some(n) => n,
                None => continue,
            };
            let param_type = match Param::from_str(file_name) {
                Ok(p) => p,
                Err(_) => continue,
            };
            params.insert(param_type, file.bytes.to_vec());
        }
        Ok(params)
    }

    // Decrypt the regulation file (AES)
    fn decrypt(cipher_text: &[u8]) -> Result<Vec<u8>, Error> {
        type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
        let key = [0x99, 0xBF, 0xFC, 0x36, 0x6A, 0x6B, 0xC8, 0xC6, 0xF5, 0x82, 0x7D, 0x09, 0x36, 0x02, 0xD6, 0x76, 0xC4, 0x28, 0x92, 0xA0, 0x1C, 0x20, 0x7F, 0xB0, 0x24, 0xD3, 0xAF, 0x4E, 0x49, 0x3F, 0xEF, 0x99];
        if cipher_text.len() < 16 {
            return Err(Error::new(std::io::ErrorKind::InvalidData, "Regulation data too short to decrypt"));
        }
        let iv = &cipher_text[0..16];
        let mut buf = cipher_text[16..cipher_text.len()].to_vec();
        let pt: &[u8] = Aes256CbcDec::new(&key.into(), iv.into())
            .decrypt_padded_mut::<NoPadding>(&mut buf)
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, format!("Failed to decrypt regulation data: {e}")))?;
        Ok(pt.to_vec())
    }

    // Decompress the decrypted regulation file (compression_type: DCX_DFLT_11000_44_9_15)
    fn decompress(bytes: &[u8]) -> Result<Vec<u8>, Error> {
        let mut br = BinaryReader::from_u8(bytes);
        br.endian = Endian::Big;

        expect_eq(br.read_bytes(4)?, b"DCX\0".as_slice(), "DCX magic")?;
        expect_eq(br.read_i32()?, 0x11000, "DCX version")?;
        expect_eq(br.read_i32()?, 0x18, "DCX header field 0x18")?;
        expect_eq(br.read_i32()?, 0x24, "DCX header field 0x24")?;
        expect_eq(br.read_i32()?, 0x44, "DCX header field 0x44")?;
        expect_eq(br.read_i32()?, 0x4c, "DCX header field 0x4c")?;

        expect_eq(br.read_bytes(4)?, b"DCS\0".as_slice(), "DCS magic")?;
        let _decompressed_size = br.read_i32()?;
        let compressed_size = br.read_i32()?;

        expect_eq(br.read_bytes(4)?, b"DCP\0".as_slice(), "DCP magic")?;
        expect_eq(br.read_bytes(4)?, b"ZSTD".as_slice(), "DCP compression type")?;
        expect_eq(br.read_i32()?, 0x20, "DCP header field 0x20")?;
        expect_eq(br.read_u8()?, 0x15, "DCP header byte 0")?;
        expect_eq(br.read_u8()?, 0, "DCP header byte 1")?;
        expect_eq(br.read_u8()?, 0, "DCP header byte 2")?;
        expect_eq(br.read_u8()?, 0, "DCP header byte 3")?;
        expect_eq(br.read_i32()?, 0, "DCP reserved i32")?;
        expect_eq(br.read_u8()?, 0, "DCP header byte 4")?;
        expect_eq(br.read_u8()?, 0, "DCP header byte 5")?;
        expect_eq(br.read_u8()?, 0, "DCP header byte 6")?;
        expect_eq(br.read_u8()?, 0, "DCP header byte 7")?;
        expect_eq(br.read_i32()?, 0, "DCP reserved i32 2")?;
        expect_eq(br.read_i32()?, 0x00010100, "DCP header field 0x00010100")?;

        expect_eq(br.read_bytes(4)?, b"DCA\0".as_slice(), "DCA magic")?;
        expect_eq(br.read_i32()?, 8, "DCA header field")?;

        let compressed = br.read_bytes(compressed_size as usize)?;

        let mut decoder = ruzstd::decoding::StreamingDecoder::new(compressed)
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, format!("Failed to init zstd decoder: {e}")))?;
        let mut decompressed = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut decompressed)
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, format!("Failed to decompress regulation data: {e}")))?;
        Ok(decompressed)
    }

    // Unpack the decrypted and decompressed regulation file (BND4)
    fn unpack(bytes: &[u8]) -> Result<BND4, Error>{
        BND4::from_bytes(bytes)
    }
}

#[cfg(test)]
mod decompress_tests {
    use super::*;

    // Real zstd-compressed data produced by the system `zstd` CLI (not the removed C-backed
    // `zstd` crate), decompressing to the 330-byte payload below. This exercises the actual
    // ruzstd decode path end-to-end through Regulation::decompress, not just "it compiles".
    const COMPRESSED_PAYLOAD: [u8; 90] = [
        0x28, 0xB5, 0x2F, 0xFD, 0x64, 0x4A, 0x00, 0x65, 0x02, 0x00, 0x82, 0xC5, 0x10, 0x11, 0xA0, 0xED,
        0xB8, 0x49, 0x65, 0xDF, 0x0A, 0xFB, 0x56, 0x0B, 0x38, 0x60, 0xD5, 0x80, 0xA8, 0xE2, 0x1A, 0x84,
        0x7C, 0xB9, 0x2C, 0x29, 0x37, 0xC2, 0xF6, 0xCC, 0x7B, 0x7D, 0x19, 0x69, 0x7F, 0xFD, 0xA6, 0xDB,
        0x8F, 0xCB, 0xBC, 0xD0, 0xE5, 0xBA, 0x5C, 0xAA, 0x0F, 0x0E, 0x79, 0x5C, 0x09, 0x7C, 0xCB, 0xED,
        0xBB, 0xB7, 0x9A, 0xF0, 0x4C, 0x1B, 0x9F, 0xDC, 0x1A, 0x77, 0x7B, 0x38, 0xFC, 0x31, 0xAF, 0x08,
        0x01, 0x00, 0xD8, 0x9B, 0x02, 0x51, 0xE9, 0x52, 0x99, 0x05,
    ];
    const EXPECTED_PAYLOAD: &[u8] = b"Hello Elden Ring regulation test payload, this is filler text to make it compressible aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn wrap_dcx(compressed: &[u8], decompressed_size: i32) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend(b"DCX\0");
        bytes.extend(0x11000i32.to_be_bytes());
        bytes.extend(0x18i32.to_be_bytes());
        bytes.extend(0x24i32.to_be_bytes());
        bytes.extend(0x44i32.to_be_bytes());
        bytes.extend(0x4ci32.to_be_bytes());

        bytes.extend(b"DCS\0");
        bytes.extend(decompressed_size.to_be_bytes());
        bytes.extend((compressed.len() as i32).to_be_bytes());

        bytes.extend(b"DCP\0");
        bytes.extend(b"ZSTD");
        bytes.extend(0x20i32.to_be_bytes());
        bytes.push(0x15);
        bytes.push(0);
        bytes.push(0);
        bytes.push(0);
        bytes.extend(0i32.to_be_bytes());
        bytes.push(0);
        bytes.push(0);
        bytes.push(0);
        bytes.push(0);
        bytes.extend(0i32.to_be_bytes());
        bytes.extend(0x00010100i32.to_be_bytes());

        bytes.extend(b"DCA\0");
        bytes.extend(8i32.to_be_bytes());

        bytes.extend(compressed);
        bytes
    }

    #[test]
    fn decompress_decodes_real_zstd_data_via_ruzstd() {
        let wrapped = wrap_dcx(&COMPRESSED_PAYLOAD, EXPECTED_PAYLOAD.len() as i32);
        let decompressed = Regulation::decompress(&wrapped).expect("should decompress successfully");
        assert_eq!(decompressed, EXPECTED_PAYLOAD);
    }
}