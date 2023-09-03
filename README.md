# Image XOR

This script uses a seeded RNG to generate a mask of noise from a provided password. That mask is then applied to a source image with the `xor` operator. This makes the result looks like noise.

Because the xor operator toggles bits on and off and the password will always generate the same noise, encryption and decryption are the same.

```
Usage: imgxor.exe <IMG_FILE> <MASK_PASSWORD> <OUT_FILE>

Arguments:
  <IMG_FILE>       Path to the source image
  <MASK_PASSWORD>  Encryption Password
  <OUT_FILE>       Path to write the output file

Options:
  -h, --help  Print help
```
