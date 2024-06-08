package main

import "github.com/containrrr/shoutrrr"
import "C"

//export Shoutrrr
func Shoutrrr(url *C.char, msg *C.char) *C.char {
	// Convert FFI C string to Go string
	url_string := C.GoString(url)
	msg_string := C.GoString(msg)

	// Call shoutrrr URL
	err := shoutrrr.Send(url_string, msg_string)

	// Convert error to string for return
	var err_str string
	if err != nil {
		err_str = err.Error()
	} else {
		err_str = ""
	}

	// Convert error string to C string for FFI
	return C.CString(err_str)
}

func main() {}
