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

extern char* ext_generate_keys(uint32_t parties, uint32_t threshold);
extern char* ext_deal_triples(size_t parties, size_t threshold);
*/
import "C"

import (
	"fmt"
)

func main() {
	example_keygen_participants()
	example_deal_triples()
}

func example_keygen_participants() C.char {
	numParticipants := C.uint32_t(2)
	numThreshold := C.uint32_t(2)
	serialized_keygen_out := C.ext_generate_keys(numParticipants, numThreshold)
	fmt.Println("Keys:", serialized_keygen_out)
	return *serialized_keygen_out
}

func example_deal_triples() C.char {
	numParticipants := C.size_t(2)
	serialized_triples := C.ext_deal_triples(numParticipants, 2)
	fmt.Println("Triples:", serialized_triples)
	return *serialized_triples
}
