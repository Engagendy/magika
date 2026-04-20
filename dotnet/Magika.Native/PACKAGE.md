# Magika.Native

Community `.NET 9` and `.NET 10` wrapper for Magika using a thin Rust native shim.

## Install

```bash
dotnet add package Engagendy.Magika.Native
```

Supported managed target frameworks:

- `net9.0`
- `net10.0`

## Package contents

- Managed wrapper assembly for `.NET 9` and `.NET 10`
- Native runtime assets when present under `runtimes/<RID>/native/`

## Native assets layout

Current published runtime targets:

- `runtimes/osx-arm64/native/libmagika_dotnet.dylib`
- `runtimes/linux-x64/native/libmagika_dotnet.so`
- `runtimes/win-x64/native/magika_dotnet.dll`

## Usage

```csharp
using Magika.Native;

using var session = new MagikaSession();
string json = session.IdentifyPathJson("/path/to/file");
Console.WriteLine(json);
```

The native shim returns UTF-8 JSON for both path and in-memory byte classification calls.

## Common examples

### Basic file-path classification

```csharp
using System.Text.Json;
using Magika.Native;

using var session = new MagikaSession();
string json = session.IdentifyPathJson("/data/invoice.pdf");

using JsonDocument document = JsonDocument.Parse(json);
string? label = document.RootElement.GetProperty("info").GetProperty("label").GetString();
string? mimeType = document.RootElement.GetProperty("info").GetProperty("mimeType").GetString();

Console.WriteLine($"{label} / {mimeType}");
```

### Classify bytes already in memory

```csharp
using Magika.Native;

byte[] bytes = await File.ReadAllBytesAsync("upload.bin");

using var session = new MagikaSession();
string json = session.IdentifyBytesJson(bytes);
Console.WriteLine(json);
```

### Deserialize to a typed DTO

```csharp
using System.Text.Json;
using System.Text.Json.Serialization;
using Magika.Native;

public sealed record MagikaResponse(
    [property: JsonPropertyName("ok")] bool Ok,
    [property: JsonPropertyName("score")] double? Score,
    [property: JsonPropertyName("info")] MagikaTypeInfo? Info,
    [property: JsonPropertyName("error")] string? Error);

public sealed record MagikaTypeInfo(
    [property: JsonPropertyName("label")] string? Label,
    [property: JsonPropertyName("mimeType")] string? MimeType);

using var session = new MagikaSession();
string json = session.IdentifyPathJson("invoice.pdf");

MagikaResponse? result = JsonSerializer.Deserialize<MagikaResponse>(json);
if (result is not null && result.Ok)
{
    Console.WriteLine($"{result.Info?.Label} / {result.Info?.MimeType} / {result.Score}");
}
```

### Reusable service wrapper

```csharp
using System.Text.Json;
using Magika.Native;

public sealed record MagikaDetection(string? Label, string? MimeType, double? Score, string RawJson);

public sealed class MagikaDetector
{
    private readonly MagikaSession _session;

    public MagikaDetector(MagikaSession session)
    {
        _session = session;
    }

    public MagikaDetection DetectBytes(byte[] bytes)
    {
        string json = _session.IdentifyBytesJson(bytes);
        using JsonDocument document = JsonDocument.Parse(json);

        return new MagikaDetection(
            document.RootElement.GetProperty("info").GetProperty("label").GetString(),
            document.RootElement.GetProperty("info").GetProperty("mimeType").GetString(),
            document.RootElement.GetProperty("score").GetDouble(),
            json);
    }
}
```

### ASP.NET upload example

```csharp
using System.Text.Json;
using Magika.Native;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddSingleton<MagikaSession>();

var app = builder.Build();

app.MapPost("/upload", async (IFormFile file, MagikaSession magika) =>
{
    await using var stream = file.OpenReadStream();
    using var memory = new MemoryStream();
    await stream.CopyToAsync(memory);

    string json = magika.IdentifyBytesJson(memory.ToArray());
    using JsonDocument document = JsonDocument.Parse(json);

    return Results.Ok(document.RootElement.Clone());
});

app.Run();
```

### ASP.NET upload validation example

```csharp
using System.Text.Json;
using Magika.Native;

var allowedMimeTypes = new HashSet<string>(StringComparer.OrdinalIgnoreCase)
{
    "image/png",
    "image/jpeg",
    "application/pdf"
};

app.MapPost("/validate-upload", async (IFormFile file, MagikaSession magika) =>
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

    string? mimeType = document.RootElement.GetProperty("info").GetProperty("mimeType").GetString();
    if (mimeType is null || !allowedMimeTypes.Contains(mimeType))
    {
        return Results.BadRequest(new
        {
            file.FileName,
            Error = "File type is not allowed.",
            DetectedMimeType = mimeType
        });
    }

    return Results.Ok(new
    {
        file.FileName,
        DetectedMimeType = mimeType
    });
});
```

## Notes

- Register `MagikaSession` as a singleton in ASP.NET instead of recreating it for every request.
- Use Magika's detected `mimeType`, `label`, and `score` for validation rather than trusting the uploaded content type header alone.
- The full cookbook, including more upload scenarios, is in the repository README: `https://github.com/Engagendy/magika`
