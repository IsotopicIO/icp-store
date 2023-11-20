use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, BTreeMap, Vec};

use std::cell::RefCell;
use std::fmt;
use std::result::Result as StdResult;
use std::collections::HashSet;
use std::collections::HashMap;

use candid::{CandidType, Principal, Deserialize};

use ic_cdk::{
    api::{self},
    init, query, update
};
type VMemory = VirtualMemory<DefaultMemoryImpl>;
type StdVec<T> = std::vec::Vec<T>;

const WASM_PAGE_SIZE: u64 = 65536;

mod types;
mod utils;


thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static OWNERS: RefCell<Vec<types::StorablePrincipal, VMemory>> = RefCell::new(
        Vec::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        ).expect("Storage expected to initiate.")
    );

    static UPLOADS: RefCell<BTreeMap<u128, Option<types::Upload>, VMemory>> = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );

    static FILE_CHUNKS: RefCell<BTreeMap<u128, StdVec<u8>, VMemory>> = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );

    static OVERWRITABLE_CHUNK_INDECES: RefCell<Vec<u128, VMemory>> = RefCell::new(
        Vec::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
        ).expect("Storage expected to initiate.")
    );

    static ISO_ID_TO_INDECES_MAP: RefCell<BTreeMap<String, types::AppUploadIDs, VMemory>> = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))),
        )
    );
}



fn check_owners(principal : Principal) -> bool{
    return OWNERS.with(|p| -> bool {
        for owner in p.borrow().iter() {
            if owner.principal == principal {
                return true;
            }
        }
        return false;
    });
}


#[derive(CandidType, Deserialize, Debug)]
enum Error {
    Unauthorized,
    Other,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

type Result<T = u128, E = Error> = StdResult<T, E>;

#[derive(CandidType, Deserialize)]
struct InitArgs {
    owners: Option<HashSet<Principal>>,
}

#[init]
fn init(args: InitArgs) {
    let _ = OWNERS.with(|p| {
        let owners =  p.borrow_mut();
        if args.owners.is_none() {
            let _ =owners.push(&types::StorablePrincipal { principal: api::caller()});
        } else {
            for owner in args.owners.unwrap().iter(){
                let _ = owners.push(&types::StorablePrincipal { principal: *owner});
            }
        }
    });
}

fn owner_only_guard() -> StdResult<(), String> {
    if check_owners(api::caller()) {
        Ok(())
    } else {
        Err(Error::Unauthorized.to_string())
    }
}

fn get_upload_by_index(upload_index: u128) -> Option<types::Upload> {
    match UPLOADS.with(|p| p.borrow().get(&upload_index)) {
        None => None,
        Some(option) => option
    }
}

#[query]
fn get_upload_chunk(upload_index: u128, chunk_index: u128) -> Option<StdVec<u8>> {
    FILE_CHUNKS.with(|p| {
        match get_upload_by_index(upload_index) {
            None => None,
            Some(upload) => upload.get_chunk(chunk_index, &p.borrow())
        }
    })
    
}

#[update(guard="owner_only_guard")]
fn upload_chunk(upload_index: u128, chunk: StdVec<u8>) -> bool {
    if chunk.len() > types::CHUNK_MAX_SIZE {
        return false;
    }
    FILE_CHUNKS.with(|file_chunks| {
        UPLOADS.with(|uploads| {
            let mut chunks = file_chunks.borrow_mut();
            let mut uploads_mut = uploads.borrow_mut();
            let upload = uploads_mut.get(&upload_index);

            match upload {
                None => false,
                Some(upload_option) => {
                    match upload_option {
                        None => false,
                        Some(mut upload_val) => {
                            let next_index = OVERWRITABLE_CHUNK_INDECES.with(|overwritable| {
                                match overwritable.borrow_mut().pop() {
                                    None => {
                                        utils::get_next_key(&chunks)
                                    },
                                    Some(index) => index
                                }
                            });

                            chunks.insert(next_index, chunk);
        
                            upload_val.chunk_indeces.push(next_index);
                            uploads_mut.insert(upload_index, Some(upload_val));
                            return true;
                        }
                    }
                }
            }
        })
        
    })
}


#[update(guard="owner_only_guard")]
fn init_new_upload(original_name: String, file_size: u128, isotopic_app_id: String, platform:String) -> u128 {
    UPLOADS.with(|p| {
        let mut uploads = p.borrow_mut();
        let index = utils::get_next_key(&uploads);
        uploads.insert(index, Some(types::Upload::new(index, original_name, file_size, isotopic_app_id, platform)));
        return index;
    })
}

#[update(guard="owner_only_guard")]
fn delete_upload(index : u128) -> bool {
    match get_upload_chunk_indeces(index){
        None => false,
        Some(indeces) => {
            OVERWRITABLE_CHUNK_INDECES.with(|p| {
                let overwritable = p.borrow_mut();
                for chunk_index in indeces.iter() {
                    let _ = overwritable.push(&chunk_index);
                }
                UPLOADS.with(|p| {
                    let mut uploads = p.borrow_mut();
                    uploads.insert(index, None);
                });
                return true;
            })
        }
    }
}

#[query(guard="owner_only_guard")]
fn get_chunk_at_stored_index(index : u128) -> Option<StdVec<u8>> {
    FILE_CHUNKS.with(|p|{
        let borrowed = p.borrow();
        borrowed.get(&index)
    })
}

#[query(guard="owner_only_guard")]
fn get_stable_memory_size() -> u64 {
    (ic_cdk::api::stable::stable64_size() as u64) * WASM_PAGE_SIZE
}

#[query(guard="owner_only_guard")]
fn get_upload_chunk_indeces(upload_index:u128) -> Option<StdVec<u128>> {
    match get_upload_by_index(upload_index) {
        None => None,
        Some(upload) => Some(upload.chunk_indeces)
    }
}

#[derive(CandidType, Deserialize)]
struct UploadDataResult {
    pub index : u128,
    pub original_name : String,
    pub file_size : u128,
    pub isotopic_app_id : String,
    pub platform : String,
    pub status : String,
    pub upload_timestamp : u64,
    pub chunks_length : u32
}


#[query]
fn get_upload_details_by_index(upload_index:u128) -> Option<UploadDataResult> {
    match get_upload_by_index(upload_index) {
        None => None,
        Some(upload) => Some(UploadDataResult {
            index: upload.index,
            original_name: upload.original_name,
            file_size: upload.file_size,
            isotopic_app_id: upload.isotopic_app_id,
            platform: upload.platform,
            status: upload.status.to_string(),
            upload_timestamp: upload.upload_timestamp,
            chunks_length : upload.chunks_length
        })
    }
}

#[update(guard="owner_only_guard")]
fn set_platform_upload(iso_app_id:String, platform:String, upload_id:Option<u128>) -> bool {
    ISO_ID_TO_INDECES_MAP.with(|p| {
        let mut borrowed = p.borrow_mut();
        match borrowed.get(&iso_app_id){
            None => match upload_id {
                None => false,
                Some(upload_id_val) => {
                    let _ = borrowed.insert(iso_app_id, types::AppUploadIDs {
                        platform_uploads: HashMap::from([
                            (platform, upload_id_val),
                        ])
                    });

                    true
                }
            },
            Some(mut uploads) => match upload_id {
                None => match uploads.platform_uploads.remove(&platform) {
                    None => false,
                    Some(_) => true
                },
                Some(upload_id_val) => {
                    uploads.platform_uploads.insert(platform, upload_id_val);
                    borrowed.insert(iso_app_id, uploads);
                    true
                } 
            }
        }
    })
}

#[update(guard="owner_only_guard")]
fn mutate_uploads_to_new_iso_id(old_iso_id:String, new_iso_id:String) -> bool {
    ISO_ID_TO_INDECES_MAP.with(|p| {
        let mut borrowed = p.borrow_mut();
        match borrowed.remove(&old_iso_id){
            None => false,
            Some(old_uploads) => {
                borrowed.insert(new_iso_id, old_uploads);

                true
            }
        }
    })
}

//candid: vec record {text, nat}
#[query]
fn get_uploads_by_iso_id(iso_id:String) -> HashMap<String, u128> {
    ISO_ID_TO_INDECES_MAP.with(|p| {
        match p.borrow().get(&iso_id) {
            None => HashMap::new(),
            Some(upload_ids) => upload_ids.platform_uploads
        }
    })
}

#[query]
fn get_chunk_max_size() -> usize {
    types::CHUNK_MAX_SIZE
}
