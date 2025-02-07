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
extern TripleResult ext_deal_triples(uint32_t parties, uint32_t threshold, uint32_t* results_participant_u32, size_t num_participants, char **results_output_serialized, size_t num_results);
*/
import "C"

import (
	"fmt"
	"unsafe"
)

func main() {
	research_keygen()
	research_deal_triples()
}

func research_keygen() {
	numParticipants := C.uint32_t(2)
	numThreshold := C.uint32_t(2)
	result := C.ext_generate_keys(numParticipants, numThreshold)
	if result.participants == nil || result.shares == nil || result.length == 0 {
		fmt.Println("Failed to generate key shares")
		return
	}
	// Convert participants array correctly - array size limited to 1 MB
	// participants := (*[1 << 18]C.uint32_t)(unsafe.Pointer(result.participants))[:result.length:result.length]
	// Convert shares array (**C.char to Go slice) - array size limited to 1 MB
	shares := make([]string, result.length)
	sharesPtr := (*[1 << 18]*C.char)(unsafe.Pointer(result.shares))[:result.length:result.length] // Convert **C.char to slice
	for i := 0; i < int(result.length); i++ {
		if sharesPtr[i] == nil {
			//fmt.Printf("Warning: NULL string at index %d\n", i)
			shares[i] = "(NULL)"
		} else {
			shares[i] = C.GoString(sharesPtr[i]) // Convert C string to Go string
		}
	}
	//fmt.Println("Final Participants [Go]:", participants)
	//fmt.Println("Shares [Go]:")
	/*for i, share := range shares {
		fmt.Printf("Participant %d: %s\n", participants[i], share)
	}*/
}

func research_deal_triples() {
	// participants should be from keygen
	participants := []C.uint32_t{101, 202, 303}
	numParticipants := C.size_t(len(participants))
	participantsPtr := (*C.uint32_t)(unsafe.Pointer(&participants[0]))
	// results should be from keygen
	results := []string{`"result1"`, `"result2"`, `"result3"`}
	numResults := C.size_t(len(results))
	// Convert Go strings to **C.char
	resultsC := make([]*C.char, len(results))
	for i, s := range results {
		resultsC[i] = C.CString(s)
	}
	// Properly pass a pointer to resultsC array
	resultsPtr := (**C.char)(unsafe.Pointer(&resultsC[0]))
	// Call the C function
	result := C.ext_deal_triples(3, 2, participantsPtr, numParticipants, resultsPtr, numResults)
	fmt.Println(result)
	// Convert C **char to Go slice of strings (triples)
	triplesJSON := C.GoString((*C.char)(unsafe.Pointer(result.triples)))
	otherTriplesJSON := C.GoString((*C.char)(unsafe.Pointer(result.other_triples)))
	fmt.Println(triplesJSON)
	fmt.Println(otherTriplesJSON)
}
