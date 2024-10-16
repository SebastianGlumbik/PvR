//! Run this file with `cargo test --test 01_read_after_eof`.

// implement `OpenedFile` in a way that it cannot be used anymore after it reaches end-of-file
// while reading.
mod os {
    // Simulate a single byte of data being read from the OS for the passed file descriptor.
    // When this function returns `0`, it marks end-of-file.
    pub fn read(fd: u32) -> u8 {
        // It doesn't matter what is actually returned here, only the function signatures are
        // important in this example.
        0
    }
}

// Library code
struct OpenedFile {
    // File descriptor
    fd: u32,
}

enum ReadResult {
    ReadByte(u8, OpenedFile),
    EndOfFile,
}

// Implement this function in a way that when the file reaches end-of-file (there is nothing
// else to read), it will not be possible to use it anymore (such usage should result in a
// compile-time error).
fn read(file: OpenedFile) -> ReadResult {
    let byte = os::read(file.fd);
    if byte != 0 {
        ReadResult::ReadByte(byte, file)
    } else {
        ReadResult::EndOfFile
    }
}
// End of library code

// User code
fn main() {
    let mut file = OpenedFile { fd: 1 };

    while let ReadResult::ReadByte(byte, new_file) = read(file) {
        file = new_file;
        println!("{}", byte);
    }
}
