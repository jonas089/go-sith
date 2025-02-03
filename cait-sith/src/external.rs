use crate::{
    keygen,
    protocol::{run_protocol, Participant, Protocol},
    KeygenOutput,
};
use k256::Secp256k1;
use std::ffi::{c_char, CString};
use std::os::raw::c_uint;

#[repr(C)]
pub struct KeygenResult {
    participants: *mut c_uint, // Pointer to array of u32
    shares: *mut *mut c_char,  // Pointer to array of C strings
    length: usize,             // Length of both arrays
}

#[no_mangle]
pub extern "C" fn ext_generate_keys(parties: u32, threshold: u32) -> KeygenResult {
    let participants: Vec<Participant> =
        (0..parties).map(|p| Participant::from(p as u32)).collect();

    let mut protocols: Vec<(
        Participant,
        Box<dyn Protocol<Output = KeygenOutput<Secp256k1>>>,
    )> = Vec::with_capacity(participants.len());

    for p in &participants {
        let protocol = keygen(&participants, *p, threshold as usize);
        assert!(protocol.is_ok());
        let protocol = protocol.unwrap();
        protocols.push((*p, Box::new(protocol)));
    }

    let keygen_out: Vec<(Participant, KeygenOutput<Secp256k1>)> = run_protocol(protocols).unwrap();

    // 🔹 Preserve Order Mapping BEFORE unzip
    let mut ordered_mapping: Vec<(u32, String)> = keygen_out
        .iter()
        .map(|(p, share)| {
            let id = u32::from(*p); // Extract participant ID
            let json = serde_json::to_string(share).unwrap();
            println!(
                "Mapping: Participant {:?} -> u32: {} -> Share: {}",
                p, id, json
            ); // Debug print
            (id, json)
        })
        .collect();

    // 🔹 Sort explicitly by participant ID
    ordered_mapping.sort_by_key(|&(id, _)| id);

    // 🔹 Extract ordered lists
    let participants_u32: Vec<u32> = ordered_mapping.iter().map(|&(id, _)| id).collect();
    let shares_serialized: Vec<CString> = ordered_mapping
        .iter()
        .map(|(_, json)| CString::new(json.clone()).unwrap())
        .collect();

    // 🔹 Preserve memory order
    let shares_ptrs: Vec<*mut c_char> = shares_serialized
        .iter()
        .map(|s| s.clone().into_raw())
        .collect();

    // 🔹 Convert to Heap Allocation
    let participants_ptr = Box::into_raw(participants_u32.into_boxed_slice()) as *mut c_uint;
    let shares_ptr = Box::into_raw(shares_ptrs.into_boxed_slice()) as *mut *mut c_char;

    KeygenResult {
        participants: participants_ptr,
        shares: shares_ptr,
        length: ordered_mapping.len(),
    }
}
