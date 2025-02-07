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
    char **shares;
    size_t length;
} KeygenResult;

typedef struct {
    char **triples;
	char **other_triples;
} TripleResult;

extern KeygenResult ext_generate_keys(uint32_t parties, uint32_t threshold);
extern TripleResult ext_deal_triples(size_t parties, uint32_t threshold, char **results_output_serialized, size_t num_results);
*/
import "C"

import (
	"fmt"
	"unsafe"
)

func main() {
	example_deal_triples()
}

func example_keygen_participants() ([]C.uint32_t, []*C.char) {
	numParticipants := C.uint32_t(2)
	numThreshold := C.uint32_t(2)
	result := C.ext_generate_keys(numParticipants, numThreshold)
	if result.participants == nil || result.shares == nil || result.length == 0 {
		fmt.Println("Failed to generate key shares")
		return nil, nil
	}
	// participants array size limited to 1 MB in this example
	participants := (*[1 << 18]C.uint32_t)(unsafe.Pointer(result.participants))[:result.length:result.length]
	// same for shares
	keysPtr := (*[1 << 18]*C.char)(unsafe.Pointer(result.shares))[:result.length:result.length] // Convert **C.char to slice
	/*for i := 0; i < int(result.length); i++ {
		if sharesPtr[i] == nil {
			//fmt.Printf("Warning: NULL string at index %d\n", i)
			shares[i] = "(NULL)"
		} else {
			shares[i] = C.GoString(sharesPtr[i]) // Convert C string to Go string
		}
	}*/
	//fmt.Println("Final Participants [Go]:", participants)
	//fmt.Println("Shares [Go]:")
	/*for i, share := range shares {
		fmt.Printf("Participant %d: %s\n", participants[i], share)
	}*/
	return participants, keysPtr
}

func example_deal_triples() (**C.char, **C.char) {
	// participants should be from keygen
	// results should be from keygen
	participants, results := example_keygen_participants()
	numResults := C.size_t(len(results))
	numParticipants := C.size_t(len(participants))
	// Properly pass a pointer to resultsC array
	resultsPtr := (**C.char)(unsafe.Pointer(&results[0]))
	// Call the C function
	result := C.ext_deal_triples(numParticipants, 2, resultsPtr, numResults)
	fmt.Println(result)
	// Convert C **char to Go slice of strings (triples)
	// triplesJSON := C.GoString((*C.char)(unsafe.Pointer(result.triples)))
	// otherTriplesJSON := C.GoString((*C.char)(unsafe.Pointer(result.other_triples)))
	fmt.Println("Triples for Participant 0:", result.triples)
	fmt.Println("Other Triples: ", result.other_triples)
	return result.triples, result.other_triples
}
