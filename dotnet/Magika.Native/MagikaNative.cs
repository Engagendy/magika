using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace Magika.Native;

/// <summary>
/// Owns a native Magika session and exposes JSON-based classification helpers.
/// </summary>
public sealed class MagikaSession : SafeHandleZeroOrMinusOneIsInvalid
{
    /// <summary>
    /// Creates a new native Magika session using the shim defaults.
    /// </summary>
    public MagikaSession() : base(true)
    {
        SetHandle(NativeMethods.magika_session_new());
        if (IsInvalid)
        {
            throw new InvalidOperationException("Failed to create Magika session.");
        }
    }

    /// <summary>
    /// Classifies a file by path and returns the native result JSON.
    /// </summary>
    public string IdentifyPathJson(string path)
    {
        if (IsClosed)
        {
            throw new ObjectDisposedException(nameof(MagikaSession));
        }

        nint jsonPtr = NativeMethods.magika_identify_path_json(handle, path);
        return NativeMethods.ConsumeJsonString(jsonPtr);
    }

    /// <summary>
    /// Classifies in-memory content and returns the native result JSON.
    /// </summary>
    public string IdentifyBytesJson(ReadOnlySpan<byte> data)
    {
        if (IsClosed)
        {
            throw new ObjectDisposedException(nameof(MagikaSession));
        }

        unsafe
        {
            fixed (byte* dataPtr = data)
            {
                nint jsonPtr = NativeMethods.magika_identify_bytes_json(handle, dataPtr, (nuint)data.Length);
                return NativeMethods.ConsumeJsonString(jsonPtr);
            }
        }
    }

    /// <summary>
    /// Releases the native Magika session handle.
    /// </summary>
    protected override bool ReleaseHandle()
    {
        NativeMethods.magika_session_free(handle);
        return true;
    }
}

internal static partial class NativeMethods
{
    private const string LibraryName = "magika_dotnet";

    [LibraryImport(LibraryName, EntryPoint = "magika_session_new")]
    internal static partial nint magika_session_new();

    [LibraryImport(LibraryName, EntryPoint = "magika_session_free")]
    internal static partial void magika_session_free(nint handle);

    [LibraryImport(LibraryName, EntryPoint = "magika_identify_path_json", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial nint magika_identify_path_json(nint handle, string path);

    [LibraryImport(LibraryName, EntryPoint = "magika_identify_bytes_json")]
    internal static unsafe partial nint magika_identify_bytes_json(nint handle, byte* data, nuint len);

    [LibraryImport(LibraryName, EntryPoint = "magika_string_free")]
    internal static partial void magika_string_free(nint value);

    internal static string ConsumeJsonString(nint value)
    {
        if (value == nint.Zero)
        {
            throw new InvalidOperationException("Native Magika call returned a null string pointer.");
        }

        try
        {
            return Marshal.PtrToStringUTF8(value) ?? throw new InvalidOperationException("Native Magika call returned invalid UTF-8.");
        }
        finally
        {
            magika_string_free(value);
        }
    }
}
