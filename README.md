# magika_dotnet

Thin native Rust shim over the official `magika` crate for consumption from `.NET 9` through `LibraryImport` / P/Invoke.

## Exported ABI

- `magika_session_new`
- `magika_session_new_with_threads`
- `magika_session_free`
- `magika_identify_path_json`
- `magika_identify_bytes_json`
- `magika_string_free`

Both identify functions return a heap-allocated UTF-8 JSON string. The caller must free it with `magika_string_free`.

## JSON shape

Success:

```json
{
  "ok": true,
  "kind": "inferred",
  "score": 0.997,
  "info": {
    "label": "python",
    "mimeType": "text/x-python",
    "group": "code",
    "description": "Python source",
    "extensions": ["py", "pyi"],
    "isText": true
  },
  "contentType": {
    "label": "python",
    "mimeType": "text/x-python",
    "group": "code",
    "description": "Python source",
    "extensions": ["py", "pyi"],
    "isText": true
  }
}
```

Error:

```json
{
  "ok": false,
  "error": "..."
}
```

## Build

Rust is required.

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Build:

```bash
cargo build --release
```

Artifacts:

- macOS: `target/release/libmagika_dotnet.dylib`
- Linux: `target/release/libmagika_dotnet.so`
- Windows: `target/release/magika_dotnet.dll`

## NuGet packaging

The packable project is [dotnet/Magika.Native/Magika.Native.csproj](./dotnet/Magika.Native/Magika.Native.csproj).

Before publishing to `nuget.org`, update `<PackageId>` and `<Authors>` in the project file to values you actually own.

Before packing, copy the compiled native artifacts into the project-local `runtimes/<RID>/native/` folders. Example:

- `dotnet/Magika.Native/runtimes/osx-arm64/native/libmagika_dotnet.dylib`
- `dotnet/Magika.Native/runtimes/linux-x64/native/libmagika_dotnet.so`
- `dotnet/Magika.Native/runtimes/win-x64/native/magika_dotnet.dll`

Pack:

```bash
dotnet pack dotnet/Magika.Native/Magika.Native.csproj -c Release
```

Outputs:

- `dotnet/Magika.Native/bin/Release/Engagendy.Magika.Native.0.1.0.nupkg`
- `dotnet/Magika.Native/bin/Release/Engagendy.Magika.Native.0.1.0.snupkg`

## GitHub Actions release

The workflow is at `.github/workflows/release.yml`.

It does three things:

- builds native binaries on GitHub-hosted runners for `linux-x64`, `win-x64`, `osx-x64`, and `osx-arm64`
- packs the NuGet package with the generated `runtimes/<RID>/native` assets
- optionally publishes the package to `nuget.org`

Before using the publish job:

1. Push this project to a real GitHub repository.
2. Revoke any NuGet API key that has been pasted into chat or logs.
3. Create a new GitHub Actions secret named `NUGET_API_KEY`.
4. Create a GitHub environment named `nuget` if you want approval gates before publishing.

The workflow publishes automatically on `v*` tags, or manually through `workflow_dispatch` when the `publish` input is set to `true`.

## .NET usage

The sample wrapper is under [dotnet/Magika.Native](./dotnet/Magika.Native).

Example:

```csharp
using Magika.Native;

using var session = new MagikaSession();
string json = session.IdentifyPathJson("/path/to/file");
Console.WriteLine(json);
```

For NuGet packaging, place the native library under RID-specific paths such as:

- `runtimes/osx-arm64/native/libmagika_dotnet.dylib`
- `runtimes/linux-x64/native/libmagika_dotnet.so`
- `runtimes/win-x64/native/magika_dotnet.dll`
