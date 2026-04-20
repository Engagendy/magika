# magika_dotnet

Thin native Rust shim over the official `magika` crate for consumption from `.NET 9` and `.NET 10` through `LibraryImport` / P/Invoke.

## Install

```bash
dotnet add package Engagendy.Magika.Native
```

Package page:

- `https://www.nuget.org/packages/Engagendy.Magika.Native`

Supported managed target frameworks:

- `net9.0`
- `net10.0`

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

## Release

NuGet packages are built and published through GitHub Actions in `.github/workflows/release.yml`.
The published package version is derived from the Git tag, so tag `v0.1.3` publishes NuGet package version `0.1.3`.

Current published runtime targets:

- `linux-x64`
- `win-x64`
- `osx-arm64`

`osx-x64` is not currently shipped because the ONNX Runtime distribution used by `ort-sys` does not provide prebuilt `x86_64-apple-darwin` binaries.

Maintainer release instructions are in [RELEASING.md](./RELEASING.md).

## .NET usage

The sample wrapper is under [dotnet/Magika.Native](./dotnet/Magika.Native).
The NuGet package readme also includes install and usage examples directly on nuget.org.

Example:

```csharp
using Magika.Native;

using var session = new MagikaSession();
string json = session.IdentifyPathJson("/path/to/file");
Console.WriteLine(json);
```

For ASP.NET uploads, prefer `IdentifyBytesJson` when you already have the file in memory, and `IdentifyPathJson` when you save uploads to disk first.

### Typed DTO example

If you do not want to work with raw `JsonDocument`, deserialize Magika's JSON into DTOs:

```csharp
using System.Text.Json;
using System.Text.Json.Serialization;
using Magika.Native;

public sealed record MagikaResponse(
    [property: JsonPropertyName("ok")] bool Ok,
    [property: JsonPropertyName("kind")] string? Kind,
    [property: JsonPropertyName("score")] double? Score,
    [property: JsonPropertyName("info")] MagikaTypeInfo? Info,
    [property: JsonPropertyName("contentType")] MagikaTypeInfo? ContentType,
    [property: JsonPropertyName("error")] string? Error);

public sealed record MagikaTypeInfo(
    [property: JsonPropertyName("label")] string? Label,
    [property: JsonPropertyName("mimeType")] string? MimeType,
    [property: JsonPropertyName("group")] string? Group,
    [property: JsonPropertyName("description")] string? Description,
    [property: JsonPropertyName("extensions")] string[]? Extensions,
    [property: JsonPropertyName("isText")] bool? IsText);

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddSingleton<MagikaSession>();

var app = builder.Build();

app.MapPost("/upload-typed", async (IFormFile file, MagikaSession magika) =>
{
    if (file.Length == 0)
    {
        return Results.BadRequest("Empty file.");
    }

    await using var stream = file.OpenReadStream();
    using var memory = new MemoryStream();
    await stream.CopyToAsync(memory);

    string json = magika.IdentifyBytesJson(memory.ToArray());
    MagikaResponse? result = JsonSerializer.Deserialize<MagikaResponse>(json);

    if (result is null)
    {
        return Results.Problem("Failed to deserialize Magika response.");
    }

    if (!result.Ok)
    {
        return Results.BadRequest(new { result.Error });
    }

    return Results.Ok(new
    {
        file.FileName,
        result.Info?.Label,
        result.Info?.MimeType,
        result.Score
    });
});

app.Run();
```

### Upload example: classify `IFormFile` in memory

This is the simplest path for a typical upload endpoint:

```csharp
using System.Text.Json;
using Magika.Native;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddSingleton<MagikaSession>();

var app = builder.Build();

app.MapPost("/upload", async (IFormFile file, MagikaSession magika) =>
{
    if (file.Length == 0)
    {
        return Results.BadRequest("Empty file.");
    }

    await using var stream = file.OpenReadStream();
    using var memory = new MemoryStream();
    await stream.CopyToAsync(memory);

    string json = magika.IdentifyBytesJson(memory.ToArray());
    using JsonDocument document = JsonDocument.Parse(json);

    bool ok = document.RootElement.GetProperty("ok").GetBoolean();
    if (!ok)
    {
        string error = document.RootElement.GetProperty("error").GetString() ?? "Magika failed.";
        return Results.BadRequest(new { error });
    }

    string? label = document.RootElement.GetProperty("info").GetProperty("label").GetString();
    string? mimeType = document.RootElement.GetProperty("info").GetProperty("mimeType").GetString();
    double score = document.RootElement.GetProperty("score").GetDouble();

    return Results.Ok(new
    {
        file.FileName,
        DetectedLabel = label,
        MimeType = mimeType,
        Score = score
    });
});

app.Run();
```

### Upload example: save first, then classify by path

Use this when your upload flow already writes a temporary file:
Assume the same `builder`, `app`, and `AddSingleton<MagikaSession>()` setup from the previous example.

```csharp
using System.Text.Json;
using Magika.Native;

app.MapPost("/upload-to-disk", async (IFormFile file, MagikaSession magika) =>
{
    if (file.Length == 0)
    {
        return Results.BadRequest("Empty file.");
    }

    string tempPath = Path.Combine(Path.GetTempPath(), $"{Guid.NewGuid():N}_{file.FileName}");

    try
    {
        await using (var output = File.Create(tempPath))
        {
            await file.CopyToAsync(output);
        }

        string json = magika.IdentifyPathJson(tempPath);
        using JsonDocument document = JsonDocument.Parse(json);

        return Results.Ok(document.RootElement.Clone());
    }
    finally
    {
        if (File.Exists(tempPath))
        {
            File.Delete(tempPath);
        }
    }
});
```

### Notes for upload flows

- Register `MagikaSession` as a singleton and reuse it across requests instead of creating a new session for every upload.
- Use the reported `mimeType`, `label`, and `score` as validation inputs, not the browser-supplied `ContentType` alone.
- Keep normal upload limits in place. Magika helps identify content, but it is not a replacement for size limits, antivirus scanning, or authorization checks.

For NuGet packaging, place the native library under RID-specific paths such as:

- `runtimes/osx-arm64/native/libmagika_dotnet.dylib`
- `runtimes/linux-x64/native/libmagika_dotnet.so`
- `runtimes/win-x64/native/magika_dotnet.dll`
