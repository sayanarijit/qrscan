<h1 align="center">
qrscan
</h1>

<p align="center">
Scan a QR code in the terminal using the system camera or a given image.
</p>

<p align="center">

<a href="https://crates.io/crates/qrscan">
<img src="https://img.shields.io/crates/v/qrscan.svg" />
</a>

<a href="https://github.com/sayanarijit/qrscan/commits">
<img src="https://img.shields.io/github/commit-activity/m/sayanarijit/qrscan" />
</a>

</p>

<p align="center">

https://user-images.githubusercontent.com/11632726/178779071-ad5ca7da-0fc3-48c1-b725-a9834db39134.mp4

</p>

### Install

```bash
cargo install --locked --force qrscan
```

### Usage

Scan via the system camera

```bash
qrscan
```

Scan a given image file

```bash
qrscan path/to/file
```

Print the QR code on the terminal

```bash
qrscan <path/to/file> --qr --no-content
```

Also print QR code metadata

```bash
qrscan <path/to/file> --metadata
```

Print the QR code as image files

```bash
qrscan <path/to/file> --qr \
  --svg path/to/out.svg \
  --png path/to/out.png \
  --jpeg path/to/out.jpeg \
  --ascii path/to/out.ascii
```

## Packaging

<a href="https://repology.org/project/qrscan/versions"><img src="https://repology.org/badge/vertical-allrepos/qrscan.svg" /></a>
