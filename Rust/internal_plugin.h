/* VChat InternalPlugin ABI 1: 两侧各自管理内存，响应由宿主复制。 */
#ifndef VCHAT_INTERNAL_PLUGIN_H
#define VCHAT_INTERNAL_PLUGIN_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
#if defined(_WIN32)
#define VCHAT_EXPORT __declspec(dllexport)
#else
#define VCHAT_EXPORT __attribute__((visibility("default")))
#endif
typedef void* VChatPluginContext;
typedef void* VChatHostContext;
typedef int32_t (*VChatWrite)(void*, const uint8_t*, size_t);
typedef struct {
    uint32_t abi_version, struct_size;
    uint64_t generation;
    VChatHostContext context;
    int32_t (*invoke)(VChatHostContext, const uint8_t*, size_t, VChatWrite, void*);
} VChatHostApi;
typedef int32_t (*VChatLifecycle)(VChatPluginContext);
typedef struct {
    uint32_t abi_version, struct_size;
    int32_t (*create)(const VChatHostApi*, const uint8_t*, size_t, VChatPluginContext*);
    VChatLifecycle start;
    int32_t (*invoke)(VChatPluginContext, const uint8_t*, size_t, VChatWrite, void*);
    VChatLifecycle quiesce, resume;
    int32_t (*save_state)(VChatPluginContext, VChatWrite, void*);
    int32_t (*restore_state)(VChatPluginContext, const uint8_t*, size_t);
    VChatLifecycle destroy;
} VChatPluginApi;
VCHAT_EXPORT const VChatPluginApi* vchat_internal_plugin_get_api(void);
#ifdef __cplusplus
}
#endif
#endif
