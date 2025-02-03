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

extern KeygenResult ext_generate_keys(uint32_t parties, uint32_t threshold);
*/
import "C"
import (
	"fmt"
	"unsafe"
)

func main() {
	research_keygen()
}

func research_keygen() {
	numParticipants := C.uint32_t(3)
	numThreshold := C.uint32_t(2)

	// Call Rust function to generate keys
	result := C.ext_generate_keys(numParticipants, numThreshold)

	// Check if result is valid
	if result.participants == nil || result.shares == nil || result.length == 0 {
		fmt.Println("Failed to generate key shares")
		return
	}

	// Convert participants array correctly - array size limited to 1 MB
	participants := (*[1 << 18]C.uint32_t)(unsafe.Pointer(result.participants))[:result.length:result.length]

	// 🔹 Print participants BEFORE processing shares
	fmt.Println("Participants in Go BEFORE processing shares:", participants)

	// Convert shares array (**C.char to Go slice) - array size limited to 1 MB
	shares := make([]string, result.length)
	sharesPtr := (*[1 << 18]*C.char)(unsafe.Pointer(result.shares))[:result.length:result.length] // Convert **C.char to slice

	for i := 0; i < int(result.length); i++ {
		if sharesPtr[i] == nil {
			fmt.Printf("Warning: NULL string at index %d\n", i)
			shares[i] = "(NULL)"
		} else {
			shares[i] = C.GoString(sharesPtr[i]) // Convert C string to Go string
		}
	}

	// 🔹 Ensure Order Consistency in Go
	fmt.Println("Final Participants in Go:", participants)
	fmt.Println("Shares in Go:")
	for i, share := range shares {
		fmt.Printf("  Participant %d: %s\n", participants[i], share)
	}
}
