use crate::{
    keygen,
    protocol::{run_protocol, Participant, Protocol},
    triples::{deal, TriplePub, TripleShare},
    KeygenOutput,
};
use k256::Secp256k1;
use rand_core::OsRng;
use std::ffi::c_char;

#[no_mangle]
pub extern "C" fn ext_generate_keys(parties: u32, threshold: u32) -> *mut c_char {
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
    let keygen_out_serialized = serde_json::to_string(&keygen_out).unwrap();
    let keygen_out_ptr = Box::into_raw(keygen_out_serialized.into_boxed_str()) as *mut c_char;
    keygen_out_ptr
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
