# Magika.Native

Community `.NET 9` wrapper for Magika using a thin Rust native shim.

## Package contents

- Managed wrapper assembly for `.NET 9`
- Native runtime assets when present under `runtimes/<RID>/native/`

## Native assets layout

Place the compiled shim binaries here before packing:

- `runtimes/osx-arm64/native/libmagika_dotnet.dylib`
- `runtimes/osx-x64/native/libmagika_dotnet.dylib`
- `runtimes/linux-x64/native/libmagika_dotnet.so`
- `runtimes/linux-arm64/native/libmagika_dotnet.so`
- `runtimes/win-x64/native/magika_dotnet.dll`

## Usage

```csharp
using Magika.Native;

using var session = new MagikaSession();
string json = session.IdentifyPathJson("/path/to/file");
Console.WriteLine(json);
```

The native shim returns UTF-8 JSON for both path and in-memory byte classification calls.
