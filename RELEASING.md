# Releasing

Maintainer-oriented packaging and publishing notes for `Engagendy.Magika.Native` targeting `.NET 9` and `.NET 10`.

## Package project

The packable project is [dotnet/Magika.Native/Magika.Native.csproj](./dotnet/Magika.Native/Magika.Native.csproj).

Before publishing to `nuget.org`, update `<PackageId>` and `<Authors>` in the project file to values you actually own.

## Runtime assets

Before packing manually, copy the compiled native artifacts into the project-local `runtimes/<RID>/native/` folders. Example:

- `dotnet/Magika.Native/runtimes/osx-arm64/native/libmagika_dotnet.dylib`
- `dotnet/Magika.Native/runtimes/linux-x64/native/libmagika_dotnet.so`
- `dotnet/Magika.Native/runtimes/win-x64/native/magika_dotnet.dll`

## Manual pack

```bash
dotnet pack dotnet/Magika.Native/Magika.Native.csproj -c Release -p:Version=0.1.3
```

Outputs:

- `dotnet/Magika.Native/bin/Release/Engagendy.Magika.Native.0.1.0.nupkg`
- `dotnet/Magika.Native/bin/Release/Engagendy.Magika.Native.0.1.0.snupkg`

## GitHub Actions workflow

The workflow is at `.github/workflows/release.yml`.

It does three things:

- builds native binaries on GitHub-hosted runners for `linux-x64`, `win-x64`, and `osx-arm64`
- packs the NuGet package with the generated `runtimes/<RID>/native` assets
- optionally publishes the package to `nuget.org`

`osx-x64` is intentionally excluded because the current `ort-sys` prebuilt distribution used by Magika does not provide `x86_64-apple-darwin` binaries. Supporting Intel macOS would require building ONNX Runtime yourself and linking it through `ORT_LIB_LOCATION`.

## GitHub setup

Before using the publish job:

1. Push this project to a real GitHub repository.
2. Revoke any NuGet API key that has been pasted into chat or logs.
3. Create a new GitHub Actions secret named `NUGET_API_KEY`.
4. Create a GitHub environment named `nuget` if you want approval gates before publishing.

## Publish behavior

The workflow publishes automatically on `v*` tags, or manually through `workflow_dispatch` when the `publish` input is set to `true`.

For tag-driven releases, the package version is derived from the tag name:

- `v0.1.3` -> NuGet package version `0.1.3`
- `v0.1.4` -> NuGet package version `0.1.4`

For manual workflow runs, provide the `version` input explicitly.
