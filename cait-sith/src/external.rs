use crate::{
    compat::scalar_hash_extffi as scalar_hash,
    keygen, presign,
    protocol::{run_protocol, Participant, Protocol},
    sign,
    triples::{deal, TriplePub, TripleShare},
    FullSignature, KeygenOutput, PresignArguments, PresignOutput,
};
use k256::Secp256k1;
use rand_core::OsRng;
use std::ffi::{c_char, CStr};

#[no_mangle]
pub extern "C" fn free_rust_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(ptr)); // Free the Rust-allocated string
    }
}

// this is a test function, split it up for each participant to run locally
#[no_mangle]
pub extern "C" fn ext_generate_keys(parties: usize, threshold: usize) -> *mut c_char {
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
    let mut keygen_out: Vec<(Participant, KeygenOutput<Secp256k1>)> =
        run_protocol(protocols).unwrap();
    keygen_out.sort_by_key(|(p, _)| *p);
    let keygen_out_serialized = serde_json::to_string(&keygen_out).unwrap();
    let keygen_out_ptr = Box::into_raw(keygen_out_serialized.into_boxed_str()) as *mut c_char;
    keygen_out_ptr
}

// this is a test function, split it up for each participant to run locally
#[no_mangle]
pub extern "C" fn ext_deal_triples(parties: usize, threshold: usize) -> *mut c_char {
    let mut participants: Vec<Participant> = vec![];
    for i in 0..parties {
        participants.push(Participant::from(i as u32));
    }

    let triples: (TriplePub<Secp256k1>, Vec<TripleShare<Secp256k1>>) =
        deal(&mut OsRng, &participants, threshold);

    let triples_serialized = serde_json::to_string(&triples).unwrap();
    let participants_ptr = Box::into_raw(triples_serialized.into_boxed_str()) as *mut c_char;
    participants_ptr
}

// this is a test function, split it up for each participant to run locally
#[no_mangle]
pub extern "C" fn ext_run_presign(
    threshold: usize,
    keys: *mut c_char,
    triples: *mut c_char,
    other_triples: *mut c_char,
) -> *mut c_char {
    let keys_str = unsafe {
        CStr::from_ptr(keys)
            .to_str()
            .expect("Failed to convert C string to Rust string")
    };
    let triples_str = unsafe {
        CStr::from_ptr(triples)
            .to_str()
            .expect("Failed to convert C string to Rust string")
    };
    let other_triples_str = unsafe {
        CStr::from_ptr(other_triples)
            .to_str()
            .expect("Failed to convert C string to Rust string")
    };
    let keys_deserialized: Vec<(Participant, KeygenOutput<Secp256k1>)> =
        serde_json::from_str(&keys_str).expect("Failed to deserialize keys");

    let triples_deserialized: (TriplePub<Secp256k1>, Vec<TripleShare<Secp256k1>>) =
        serde_json::from_str(&triples_str).expect("Failed to deserialize triples");

    let other_triples_deserialized: (TriplePub<Secp256k1>, Vec<TripleShare<Secp256k1>>) =
        serde_json::from_str(&other_triples_str).expect("Failed to deserialize other triples");

    let mut protocols: Vec<(
        Participant,
        Box<dyn Protocol<Output = PresignOutput<Secp256k1>>>,
    )> = Vec::with_capacity(keys_deserialized.len());
    let participant_list: Vec<Participant> = keys_deserialized.iter().map(|(p, _)| *p).collect();
    for (((p, keygen_out), share0), share1) in keys_deserialized
        .into_iter()
        .zip(triples_deserialized.1.into_iter())
        .zip(other_triples_deserialized.1.into_iter())
    {
        let protocol = presign(
            &participant_list,
            p,
            PresignArguments {
                triple0: (share0, triples_deserialized.0.clone()),
                triple1: (share1, other_triples_deserialized.0.clone()),
                keygen_out,
                threshold,
            },
        );
        assert!(protocol.is_ok());
        let protocol = protocol.unwrap();
        protocols.push((p, Box::new(protocol)));
    }

    let mut presign_out: Vec<(Participant, PresignOutput<Secp256k1>)> =
        run_protocol(protocols).unwrap();
    presign_out.sort_by_key(|(p, _)| *p);
    let presign_out_serialized = serde_json::to_string(&presign_out).unwrap();
    let presign_out_ptr = Box::into_raw(presign_out_serialized.into_boxed_str()) as *mut c_char;
    presign_out_ptr
}

// this is a test function, split it up for each participant to run locally
#[no_mangle]
pub extern "C" fn ext_run_sign(
    idx: usize,
    presign_out: *mut c_char,
    keygen_out: *mut c_char,
    msg: *mut c_char,
) -> *mut c_char {
    let presign_str = unsafe {
        CStr::from_ptr(presign_out)
            .to_str()
            .expect("Failed to convert C string to Rust string")
    };
    let keygen_str = unsafe {
        CStr::from_ptr(keygen_out)
            .to_str()
            .expect("Failed to convert C string to Rust string")
    };

    let msg_str = unsafe {
        CStr::from_ptr(msg)
            .to_str()
            .expect("Failed to convert C string to Rust string")
    };
    let presign_deserialized: Vec<(Participant, PresignOutput<Secp256k1>)> =
        serde_json::from_str(&presign_str).expect("Failed to deserialize presign");
    let keygen_deserialized: Vec<(Participant, KeygenOutput<Secp256k1>)> =
        serde_json::from_str(&keygen_str).expect("Failed to deserialize keygen");
    // use the public key at the target index
    let public_key = keygen_deserialized.get(idx).unwrap().1.public_key;
    let mut protocols: Vec<(
        Participant,
        Box<dyn Protocol<Output = FullSignature<Secp256k1>>>,
    )> = Vec::with_capacity(presign_deserialized.len());
    let participant_list: Vec<Participant> = presign_deserialized.iter().map(|(p, _)| *p).collect();

    for (p, presign_out) in presign_deserialized.into_iter() {
        let protocol = sign(
            &participant_list,
            p,
            public_key,
            presign_out,
            scalar_hash(&serde_json::to_vec(&msg_str).expect("Failed to serialize message")),
        );
        assert!(protocol.is_ok());
        let protocol = protocol.unwrap();
        protocols.push((p, Box::new(protocol)));
    }

    let sign_out: Vec<(Participant, FullSignature<Secp256k1>)> = run_protocol(protocols).unwrap();
    let sign_out_serialized =
        serde_json::to_string(&sign_out).expect("Failed to serialize signature");
    let sign_out_ptr = Box::into_raw(sign_out_serialized.into_boxed_str()) as *mut c_char;
    sign_out_ptr
}
