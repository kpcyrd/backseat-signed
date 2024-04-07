# backseat-signed

Authenticate the cryptographic chain-of-custody of Linux distributions (like Arch Linux and Debian) to their source code inputs. This is done by following cryptographic links from a signed derivate to its source input.

This concept is somewhat goofy but has some interesting properties - software releases are typically signed like this:

```
example-0.1.0.tar.gz <- example-0.1.0.tar.gz.sig
```

*✨Fabulous✨*. However, the luxury of this simplicity may not always be available, upstream may not be signing their releases, or they are signing some intermediate build artifact instead of the actual source code <sup>\*hint hint\*</sup>.

Now what if this is not available? May I present you this alternative chain:

```
example-0.1.0.tar.gz <- PKGBUILD <- .BUILDINFO <- .pkg.tar.zst <- .pkg.tar.zst.sig
```

Due to a chain of happy coincidents, when an Arch Linux package maintainer signs a package they built from `example-0.1.0.tar.gz`, they sign something that contains a hash (`.pkg.tar.zst/.BUILDINFO`) of something that contains a hash (`PKGBUILD`) of the original `example-0.1.0.tar.gz`.

Or how about this one?

```
example-0.1.0.tar <- example_0.1.0.orig.tar.xz <- Sources.xz <- Release <- Release.gpg
```

This may require some squinting since in Debian the source tarball is sometimes recompressed so only the inner .tar is compared, the outer compression layer is disregarded.

## But didn't this just go wrong?

Indeed, you can use `backseat-signed` to verify `xz-5.6.1.tar.gz` <sup><sup>(`sha256:2398f4a8e53345325f44bdd9f0cc7401bd9025d736c6d43b372f4dea77bf75b8`)</sup></sup> has been in both Debian and Arch Linux.

But this is specifically why the xz thing was such a big deal.

Both have used something that wasn't a VCS snapshot and instead used an archive with source code pre-processed by autotools (and some manual changes), which is arguably an intermediate build artifact.

Since both distributions intend to build from source (with different levels of strictness), they should prefer a VCS snapshot that was taken with e.g. `git archive`.

Ideally you could:

```
git -C source-code/ -c tar.tar.gz.command="gzip -cn" archive --prefix="example-0.1.0/" -o "example-0.1.0.tar.gz" "v0.1.0"
backseat-signed verify --todo ./debian.todo example-0.1.0.tar.gz
backseat-signed verify --todo ./archlinux.todo example-0.1.0.tar.gz
```

Which hopefully makes it fairly obvious what's the soure code that people should be code reviewing. 🦝

This could then be topped off with [reproducible builds](https://reproducible-builds.org/) to verify the path from `source -> binary` too (in its entirety).

## How to use the plumbing commands

For Arch Linux:

```sh
# prepare what we want to compare with
git clone 'https://github.com/abishekvashok/cmatrix'
git -C cmatrix/ -c tar.tar.gz.command="gzip -cn" archive --prefix="cmatrix-2.0/" -o "cmatrix-2.0.tar.gz" "v2.0"

# for the lack of a better keyring file
# verify cmatrix-2.0-3-x86_64.pkg.tar.zst.sig -> cmatrix-2.0-3-x86_64.pkg.tar.zst
wget 'https://archive.archlinux.org/packages/c/cmatrix/cmatrix-2.0-3-x86_64.pkg.tar.zst'{,.sig}
backseat-signed plumbing archlinux-pkg-from-sig --keyring /usr/share/pacman/keyrings/archlinux.gpg --sig cmatrix-2.0-3-x86_64.pkg.tar.zst.sig cmatrix-2.0-3-x86_64.pkg.tar.zst
# verify cmatrix-2.0-3-x86_64.pkg.tar.zst -> PKGBUILD
wget 'https://gitlab.archlinux.org/archlinux/packaging/packages/cmatrix/-/raw/2.0-3/PKGBUILD'
backseat-signed plumbing archlinux-pkgbuild-from-pkg --pkg cmatrix-2.0-3-x86_64.pkg.tar.zst PKGBUILD
# verify PKGBUILD -> cmatrix-2.0.tar.gz
backseat-signed plumbing archlinux-file-from-pkgbuild --pkgbuild PKGBUILD cmatrix-2.0.tar.gz
```

For Debian:

```sh
# prepare what we want to compare with
git clone 'https://github.com/abishekvashok/cmatrix'
git -C cmatrix/ -c tar.tar.gz.command="gzip -cn" archive --prefix="cmatrix-2.0/" -o "cmatrix-2.0.tar.gz" "v2.0"

# verify Release.gpg -> Release -> Sources.xz
backseat-signed plumbing debian-sources-from-release --keyring debian-archive-bookworm-automatic.asc --sig Release.gpg --release Release Sources.xz
# verify Sources.xz -> cmatrix-2.0.tar.gz
# if debian recompressed your file, you need to provide this file too with `--orig cmatrix_2.0.orig.tar.xz`
backseat-signed plumbing debian-tarball-from-sources --sources Sources.xz cmatrix-2.0.tar.gz
```

> [!IMPORTANT]
> This tool is still experimental and some things are hard-coded that you'd expect to be more flexible. If something fails please open a github issue. 🖤

## Where does the name come from?

The name is derived from 'backseat gaming,' which refers to someone who is not actively playing (but merely watching) who involves themselves in a game someone else is playing.

## Why use the past tense '-signed'?

This tool does not create signatures, it only collects and verifies them.

## How does this program decompress xz/lzma?

It's using the [lzma-rs](https://crates.io/crates/lzma-rs) crate, a pure Rust reimplementation of lzma.

## Credits

This software was brought to you by humans associating with the European left-autonomous hacking scene of the 2020's, in response to the [xz backdoor incident](https://www.openwall.com/lists/oss-security/2024/03/29/4). It aims to make open source authority transparent enough to allow reasoning about them as common goods.

This project operates without public funding through [anarcho-communism](https://en.wikipedia.org/wiki/Anarchist_communism).

## License

`GPL-3.0-or-later`
