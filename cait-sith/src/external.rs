use crate::{
    keygen,
    protocol::{run_protocol, Participant, Protocol},
    triples::{self, deal, TriplePub, TripleShare},
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
    participants: *mut c_uint,     // Pointer to array of int
    keygen_outs: *mut *mut c_char, // Pointer to array of C strings
    length: usize,
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
    assert_eq!(
        keygen_out.first().unwrap().1.public_key,
        keygen_out.get(1).unwrap().1.public_key
    );
    KeygenResult {
        participants: participants_ptr,
        keygen_outs: keygen_outs_ptr,
        length: ordered_mapping.len(),
    }
}

#[no_mangle]
pub extern "C" fn ext_deal_triples(parties: usize, threshold: usize) -> *mut c_char {
    let mut participants: Vec<Participant> = vec![];
    for i in 0..parties {
        participants.push(Participant::from(i as u32));
    }

    let tripes: (TriplePub<Secp256k1>, Vec<TripleShare<Secp256k1>>) =
        deal(&mut OsRng, &participants, threshold);

    let triples_serialized = serde_json::to_string(&tripes).unwrap();
    let participants_ptr = Box::into_raw(triples_serialized.into_boxed_str()) as *mut c_char;
    participants_ptr
}

#[no_mangle]
pub extern "C" fn run_presign() {}

#[no_mangle]
pub extern "C" fn run_sign() {}
