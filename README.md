This is my Rust learning project. It is meant to store encrypted data hidden in a png file and decrypt it. The PNG file should still be readable by most image viewers.
Encryption is done using AES and Argon2 with a random salt as KDF.

Please note that since this is a learning project, code will not be perfect and expect bugs!

# Usage

    cargo build --release


    ./encoder \<password\> \<infile\> \<payload\> \<outfile\>

    ./decoder \<password\> \<encrypted png\> \<payload outfile\>