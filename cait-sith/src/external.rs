use crate::{
    keygen,
    protocol::{run_protocol, Participant, Protocol},
    triples::{self, TriplePub, TripleShare},
    KeygenOutput,
};
use k256::Secp256k1;
use rand_core::OsRng;
use std::os::raw::c_uint;
use std::{
    collections::HashMap,
    ffi::{c_char, CString},
};

#[repr(C)]
pub struct KeygenResult {
    participants: *mut c_uint,     // Pointer to array of u32
    keygen_outs: *mut *mut c_char, // Pointer to array of C strings
    length: usize,                 // Length of both arrays
}

#[repr(C)]
pub struct TripleResult {
    triples: *mut c_char,
    other_triples: *mut c_char,
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
    let mut ordered_mapping: Vec<(u32, String)> = keygen_out
        .iter()
        .map(|(p, keygen_out)| {
            let id = u32::from(*p);
            let json = serde_json::to_string(keygen_out).unwrap();
            (id, json)
        })
        .collect();
    ordered_mapping.sort_by_key(|&(id, _)| id);
    let participants_u32: Vec<u32> = ordered_mapping.iter().map(|&(id, _)| id).collect();
    let keygen_outs_serialized: Vec<CString> = ordered_mapping
        .iter()
        .map(|(_, json)| CString::new(json.clone()).unwrap())
        .collect();
    let keygen_outs_ptrs: Vec<*mut c_char> = keygen_outs_serialized
        .iter()
        .map(|s| s.clone().into_raw())
        .collect();
    let participants_ptr = Box::into_raw(participants_u32.into_boxed_slice()) as *mut c_uint;
    let keygen_outs_ptr = Box::into_raw(keygen_outs_ptrs.into_boxed_slice()) as *mut *mut c_char;
    KeygenResult {
        participants: participants_ptr,
        keygen_outs: keygen_outs_ptr,
        length: ordered_mapping.len(),
    }
}

#[no_mangle]
pub extern "C" fn ext_deal_triples(
    parties: usize,
    threshold: u32,
    results_output_serialized: *const *const c_char,
    num_results: usize,
) -> TripleResult {
    let mut participants: Vec<u32> = vec![];
    for i in 0..parties {
        participants.push(i as u32);
    }
    let serialized_outputs: Vec<String> = unsafe {
        (0..num_results)
            .map(|i| {
                let c_str = std::ffi::CStr::from_ptr(*results_output_serialized.add(i));
                c_str.to_string_lossy().into_owned()
            })
            .collect()
    };
    let results = participants.iter().zip(serialized_outputs);
    let participants: Vec<_> = (0..parties).map(|p| Participant::from(p as u32)).collect();
    let triples: HashMap<_, _> = results.into_iter().map(|(p, out)| (p, out)).collect();
    let other_triples: (TriplePub<Secp256k1>, Vec<TripleShare<Secp256k1>>) =
        triples::deal(&mut OsRng, &participants, threshold as usize);
    let triples_serialized = serde_json::to_string(&triples).unwrap();
    let triples_ptr = Box::into_raw(triples_serialized.clone().into_boxed_str()) as *mut c_char;
    let other_triples_serialized = serde_json::to_string(&other_triples).unwrap();
    let other_triples_ptr =
        Box::into_raw(other_triples_serialized.clone().into_boxed_str()) as *mut c_char;
    TripleResult {
        triples: triples_ptr as *mut c_char,
        other_triples: other_triples_ptr as *mut c_char,
    }
}
