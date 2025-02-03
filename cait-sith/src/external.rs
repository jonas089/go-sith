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

    let mut ordered_mapping: Vec<(u32, String)> = keygen_out
        .iter()
        .map(|(p, share)| {
            let id = u32::from(*p);
            let json = serde_json::to_string(share).unwrap();
            println!(
                "Mapping: Participant {:?} -> u32: {} -> Share: {}",
                p, id, json
            );
            (id, json)
        })
        .collect();

    ordered_mapping.sort_by_key(|&(id, _)| id);
    let participants_u32: Vec<u32> = ordered_mapping.iter().map(|&(id, _)| id).collect();
    let shares_serialized: Vec<CString> = ordered_mapping
        .iter()
        .map(|(_, json)| CString::new(json.clone()).unwrap())
        .collect();

    let shares_ptrs: Vec<*mut c_char> = shares_serialized
        .iter()
        .map(|s| s.clone().into_raw())
        .collect();

    let participants_ptr = Box::into_raw(participants_u32.into_boxed_slice()) as *mut c_uint;
    let shares_ptr = Box::into_raw(shares_ptrs.into_boxed_slice()) as *mut *mut c_char;

    KeygenResult {
        participants: participants_ptr,
        shares: shares_ptr,
        length: ordered_mapping.len(),
    }
}

#[no_mangle]
pub extern "C" fn ext_deal_triples(
    parties: u32,
    threshold: u32,
    results_participant_u32: *const u32,
    num_participants: usize,
    results_output_serialized: *const *const c_char,
    num_results: usize,
) {
    let participants: &[u32] =
        unsafe { std::slice::from_raw_parts(results_participant_u32, num_participants) };
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
    let (other_triples_pub, other_triples_share): (
        TriplePub<Secp256k1>,
        Vec<TripleShare<Secp256k1>>,
    ) = triples::deal(&mut OsRng, &participants, threshold as usize);
    let other_triples: HashMap<_, _> = participants
        .iter()
        .zip(other_triples_share)
        .map(|(p, share)| (p, (share, other_triples_pub.clone())))
        .collect();
    // todo: return (triples, other_triples) in a way that is FFI friendly
}
