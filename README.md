# qrscan

Scan a QR code in the terminal using the system camera or a given image.

https://user-images.githubusercontent.com/11632726/178779071-ad5ca7da-0fc3-48c1-b725-a9834db39134.mp4

### Install

```bash
cargo install --git https://github.com/sayanarijit/qrscan
```

### Usage

Scan via the system camera

```bash
qrscan
```

Scan a given image file

```bash
qrscan /path/to/image
```

Print the QR code on the terminal

```bash
qrscan </path/to/file> --qr --no-content
```

Also print QR code metadata

```bash
qrscan </path/to/file> --metadata
```
