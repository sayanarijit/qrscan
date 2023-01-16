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

[![Packaging status](https://repology.org/badge/vertical-allrepos/qrscan.svg)](https://repology.org/project/qrscan/versions)

#### Scoop

```powershell
scoop bucket add extras
scoop install qrscan
```

#### Cargo

```bash
# On Ubuntu, first install libclang-dev (apt-get install libclang-dev)
cargo install --locked --force qrscan
```

#### Pre-built binaries

Download the appropriate binary from the [latest release](https://github.com/sayanarijit/qrscan/releases/latest).

### Usage

Scan via the system camera with terminal preview

```bash
qrscan --preview
```

Scan a given image file

```bash
qrscan path/to/file

# Or read from stdin

cat /path/to/file | qrscan -
```

Print the QR code on the terminal

```bash
qrscan <path/to/file> --qr --no-content
```

Also print QR code metadata

```bash
qrscan <path/to/file> --metadata
```

Export the QR code as image files

```bash
qrscan <path/to/file> --qr \
  --svg path/to/out.svg \
  --png path/to/out.png \
  --jpeg path/to/out.jpeg \
  --ascii path/to/out.ascii
```

### Some Usage Examples

Capture a screenshot of a selected area using [ImageMagic](https://imagemagick.org/index.php) and scan the QR code.

```bash
import png:- | qrscan -
```

### Introductions & Reviews

- [Scan QR codes right from your Linux Terminal ~ The Lunduke Journal](https://lunduke.substack.com/p/scan-qr-codes-right-from-your-linux)
