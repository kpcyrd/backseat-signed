# backseat-signed

Authenticate cryptographic links from a signed derivate to its source input.

This concept is somewhat silly but has some interesting properties - software releases are typically signed like this:

```
example-0.1.0.tar.gz <- example-0.1.0.tar.gz.sig
```

/✨Fabulous✨/. However, the luxury of this simplicity may not always be available, upstream may not be signing their releases, or they are signing some intermediate build artifact instead of the actual source code <sup>\*hint hint\*</sup>.

Now what if this is not available? May I present you this alternative chain:

```
example-0.1.0.tar.gz <- PKGBUILD <- .BUILDINFO <- .pkg.tar.zst <- .pkg.tar.zst.sig
```

Due to a chain of lucky coincident, when an Arch Linux package maintainer signs a package they built from `example-0.1.0.tar.gz`, they sign something that contains a hash (`.pkg.tar.zst/.BUILDINFO`) of something that contains a hash (`PKGBUILD`) of the original `example-0.1.0.tar.gz`.

Or how about this one?

```
example-0.1.0.tar.gz <- example_0.1.0.orig.tar.xz <- Sources.gz <- Release <- Release.gpg
```

This is technically not a "bit-for-bit" chain, the source tarball is commonly recompressed so only the inner .tar is compared, the outer compression layer is disregarded.

## But didn't this just go wrong?

Indeed, you can use `backseat-signed` to verify `xz-5.6.1.tar.gz` (`sha256:2398f4a8e53345325f44bdd9f0cc7401bd9025d736c6d43b372f4dea77bf75b8`) has been in both Debian and Arch Linux.

But this is specifically why the xz thing is such a big deal.

Both have used something that wasn't a VCS snapshot and instead used an archive source code pre-processed with autotools (and some manual changes).

Since both distributions intend to build from source (with different levels of strictness), they should prefer a VCS snapshot that was taken with e.g. `git archive`.

Ideally you could:

```
git -C sourcode/ archive --prefix="example-0.1.0/" -o "example-0.1.0.tar.gz" "v0.1.0"
backseat-signed verify --todo ./debian.todo example-0.1.0.tar.gz
backseat-signed verify --todo ./archlinux.todo example-0.1.0.tar.gz
```

To verify the VCS snapshot (that you're about to review) matches the source code inputs used for both Debian and Arch Linux packages.

If this doesn't work:

- Investigate the diff of the distros source inuts (if any)
- Investigate the diff of the VCS snapshot and the distro source inputs

You should only code review from version control system if you verified this was indeed the source code used for the binaries you care about.

This could then be topped off with [reproducible builds](https://reproducible-builds.org/) to verify the path from `source -> binary` too.

## Why use the past tense '-signed'?

This tool does not create signatures, it only collects and verifies them.

## Credits

This software was brought to you by humans associating with the European left-autonomous hacking scene of the 2020's, in response to the [xz backdoor incident](https://www.openwall.com/lists/oss-security/2024/03/29/4). It aims to make open source authority transparent enough to allow reasoning about them as common goods.

This project operates without public funding through [anarcho-communism](https://en.wikipedia.org/wiki/Anarchist_communism).

## License

`GPL-3.0-or-later`
