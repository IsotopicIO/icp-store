use ic_stable_structures::memory_manager::{VirtualMemory};
use ic_stable_structures::{storable::Bound, DefaultMemoryImpl, BTreeMap, Storable};

use ic_cdk;

use std::fmt;
use std::borrow::Cow;
use std::collections::HashMap;

use candid::{CandidType, Encode, Decode, Principal, Deserialize};

type VMemory = VirtualMemory<DefaultMemoryImpl>;
type StdVec<T> = std::vec::Vec<T>;

pub static CHUNK_MAX_SIZE: usize = 1_048_576;

#[path = "utils.rs"] mod utils;

#[derive(CandidType, Deserialize)]
pub struct StorablePrincipal {
    pub principal: Principal
}

impl Storable for StorablePrincipal {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 128,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize)]
pub struct AppUploadIDs {
    pub platform_uploads : HashMap<String,u128>
}

impl Storable for AppUploadIDs {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Debug)]
pub enum UploadStatus {
    Init,
    Uploading,
    Ready,
    Unavailable,
}
impl fmt::Display for UploadStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(CandidType, Deserialize)]
pub struct Upload {
    pub index : u128,
    pub original_name : String,
    pub file_size : u128,
    pub chunk_indeces : StdVec<u128>,
    pub chunks_length : u32,
    pub isotopic_app_id : String,
    pub platform : String,
    pub status : UploadStatus,
    pub upload_timestamp : u64 //ic_cdk::api::time 
}

impl Upload {
    pub fn get_chunk(&self, index : u128, chunks : &BTreeMap<u128, StdVec<u8>, VMemory>) -> Option<StdVec<u8>>{
        if self.chunk_indeces.len() as u128 <= index {
            return None;
        }
        
        let stored_index = self.chunk_indeces[index as usize];
        return chunks.get(&stored_index);
    }

    pub fn new(index:u128, original_name:String, file_size:u128, isotopic_app_id:String, platform:String) -> Self {
        Upload {
            index: index,
            original_name: original_name,
            file_size: file_size,
            chunk_indeces: vec![],
            chunks_length : (file_size / (CHUNK_MAX_SIZE as u128) + 1) as u32,
            isotopic_app_id: isotopic_app_id,
            platform: platform,
            status: UploadStatus::Init,
            upload_timestamp: ic_cdk::api::time()
        }
    }
}

impl Storable for Upload {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}