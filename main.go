/*
	This file contains research functions that aim to wrap the core functionality of the
	cait_sith library in Go.

	This code can be re-factored and integrated to use cait_sith in a complex Go environment.
*/

package main

/*
#cgo LDFLAGS: -L./cait-sith/target/release -lexternal_cait
#cgo CFLAGS: -I.
#include <stdint.h>
#include <stdlib.h>

typedef struct {
    uint32_t *participants;
    char **keygen_out;
    size_t length;
} KeygenResult;

extern KeygenResult ext_generate_keys(uint32_t parties, uint32_t threshold);
extern char* ext_deal_triples(size_t parties, size_t threshold);
*/
import "C"

import (
	"fmt"
	"unsafe"
)

func main() {
	example_deal_triples()
}

func example_keygen_participants() []*C.char {
	numParticipants := C.uint32_t(2)
	numThreshold := C.uint32_t(2)
	result := C.ext_generate_keys(numParticipants, numThreshold)
	if result.participants == nil || result.keygen_out == nil || result.length == 0 {
		fmt.Println("Failed to generate key keygen_out")
		return nil
	}
	// same for keygen_out
	keysPtr := (*[1 << 18]*C.char)(unsafe.Pointer(result.keygen_out))[:result.length:result.length] // Convert **C.char to slice
	/*for i := 0; i < int(result.length); i++ {
		if keygen_outPtr[i] == nil {
			//fmt.Printf("Warning: NULL string at index %d\n", i)
			keygen_out[i] = "(NULL)"
		} else {
			keygen_out[i] = C.GoString(keygen_outPtr[i]) // Convert C string to Go string
		}
	}*/
	//fmt.Println("Final Participants [Go]:", participants)
	//fmt.Println("keygen_out [Go]:")
	/*for i, share := range keygen_out {
		fmt.Printf("Participant %d: %s\n", participants[i], share)
	}*/
	return keysPtr
}

func example_deal_triples() C.char {
	// participants should be from keygen
	// results should be from keygen
	//keys := example_keygen_participants()
	//numResults := C.size_t(len(keys))
	numParticipants := C.size_t(2)
	//resultsPtr := (**C.char)(unsafe.Pointer(&keys[0]))
	// Call the C function
	serialized_triples := C.ext_deal_triples(numParticipants, 2)
	// Convert C **char to Go slice of strings (triples)
	// triplesJSON := C.GoString((*C.char)(unsafe.Pointer(result.triples)))
	// otherTriplesJSON := C.GoString((*C.char)(unsafe.Pointer(result.other_triples)))
	fmt.Println("Triples:", serialized_triples)
	return *serialized_triples
}
