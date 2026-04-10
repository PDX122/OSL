# OSL Package Install Layout

This repo now supports a basic `oslc install <package>` flow that expects a zip archive from the registry.

## Registry Endpoint

The CLI downloads packages with:

```text
GET https://oslc.dev/api/packages/download?name=<package-name>
```

## On-Disk Layout

Packages install under:

```text
~/.oslc/packages/
  <package-name>/
```

For example, `oslc install help/example` maps to:

```text
~/.oslc/packages/help/example/
```

## Expected Archive Behavior

- The downloaded file should be a zip archive.
- The archive is unpacked into the package directory.
- Any previous install at that path is removed before unpacking.
- The temporary zip is deleted after a successful unpack.

## Local Requirements

This implementation shells out to:

- `curl` for downloading
- `unzip` for extraction

If you want this to be fully self-contained later, the next step is to replace those calls with Rust crates for HTTP and zip extraction.
