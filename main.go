package main

/*
#cgo LDFLAGS: -L./cait-sith/target/release -lexternal_cait
#cgo CFLAGS: -I.
#include <stdint.h>
#include <stdlib.h>

extern char* ext_generate_keys(size_t parties, size_t threshold);
extern char* ext_deal_triples(size_t parties, size_t threshold);
extern void ext_run_presign(size_t threshold, char* keys, char* triples, char* other_triples);
extern void free_rust_string(char* ptr);
*/
import "C"
import (
	"unsafe"
)

// Convert C string to Go string and free memory
func cStrToGoString(cstr *C.char) string {
	if cstr == nil {
		return ""
	}
	defer C.free_rust_string(cstr) // Ensure we free memory allocated by Rust
	return C.GoString(cstr)
}

func example_keygen_participants() string {
	numParticipants := C.size_t(2)
	numThreshold := C.size_t(2)
	cKeys := C.ext_generate_keys(numParticipants, numThreshold)
	keys := cStrToGoString(cKeys)
	return keys
}

func example_deal_triples() string {
	numParticipants := C.size_t(2)
	cTriples := C.ext_deal_triples(numParticipants, 2)
	triples := cStrToGoString(cTriples)
	return triples
}

func example_presign() {
	numThreshold := C.size_t(2)
	keys := example_keygen_participants()
	triples := example_deal_triples()
	otherTriples := example_deal_triples()

	cKeys := C.CString(keys)
	cTriples := C.CString(triples)
	cOtherTriples := C.CString(otherTriples)
	defer C.free(unsafe.Pointer(cKeys))
	defer C.free(unsafe.Pointer(cTriples))
	defer C.free(unsafe.Pointer(cOtherTriples))

	C.ext_run_presign(numThreshold, cKeys, cTriples, cOtherTriples)
}

func main() {
	example_presign()
}
