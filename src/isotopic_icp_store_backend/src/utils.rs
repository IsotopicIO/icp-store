use ic_stable_structures::{BTreeMap, Storable, Memory};

pub fn get_next_key<T, M>(map: &BTreeMap<u128, T, M>) -> u128 where M: Memory, T: Storable{
    match map.iter().last() {
        Some((last_key, _)) => last_key + 1,
        None => 0,
    }
}