#ifndef MAGIKA_DOTNET_H
#define MAGIKA_DOTNET_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MagikaSessionHandle MagikaSessionHandle;

MagikaSessionHandle* magika_session_new(void);
MagikaSessionHandle* magika_session_new_with_threads(
    size_t inter_threads,
    size_t intra_threads,
    bool parallel_execution);
void magika_session_free(MagikaSessionHandle* handle);

char* magika_identify_path_json(MagikaSessionHandle* handle, const char* path);
char* magika_identify_bytes_json(
    MagikaSessionHandle* handle,
    const uint8_t* data,
    size_t len);
void magika_string_free(char* value);

#ifdef __cplusplus
}
#endif

#endif
