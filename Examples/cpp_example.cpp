// C++ SDK 示例：直接使用结构化 C ABI，所有输出在同步 sink 中复制。
#include "VCPluginInterface.h"
#include <cstddef>
#include <limits>
#include <stdexcept>
#include <string>
#include <vector>

static_assert(offsetof(VCHostApi, generation) == 8);
static_assert(offsetof(VCPluginApi, create) == 8);
static_assert(sizeof(VCPluginApi) == 8 + sizeof(void*) * 8);
struct Context { VCHostApi host; uint64_t count = 0; };

template<class T> static T initialized() {
    T value{};
    value.header = {VC_ABI_VERSION, static_cast<uint32_t>(sizeof(T))};
    return value;
}
static VCString text(const std::string& value) {
    return {reinterpret_cast<const uint8_t*>(value.data()), static_cast<uint64_t>(value.size())};
}
static VCBytes bytes(const std::string& value) {
    return {reinterpret_cast<const uint8_t*>(value.data()), static_cast<uint64_t>(value.size()), 1};
}
static std::string copied(VCString value) {
    if (value.len > VC_MAX_BYTES || (value.len && !value.data)) throw std::invalid_argument("string view");
    return value.len ? std::string(reinterpret_cast<const char*>(value.data), static_cast<size_t>(value.len)) : std::string{};
}
// 集合按宿主提供的 stride 访问，尾部扩展保持各元素起始地址正确。
template<class T, class Slice> static std::vector<T> copied_slice(Slice value) {
    if (value.len && (!value.data || value.stride < sizeof(T) || value.stride % alignof(T))) throw std::invalid_argument("slice view");
    if (value.len > VC_MAX_BYTES / sizeof(T) || (value.len && value.stride > VC_MAX_BYTES / value.len)) throw std::invalid_argument("slice limit");
    std::vector<T> result;
    for (uint64_t i = 0; i < value.len; ++i) {
        auto item = reinterpret_cast<const T*>(reinterpret_cast<const uint8_t*>(value.data) + i * value.stride);
        if (item->header.abi_version != VC_ABI_VERSION || item->header.struct_size < sizeof(T)) throw std::invalid_argument("slice header");
        result.push_back(*item);
    }
    return result;
}
static int32_t VC_CALL create(const VCHostApi* host, VCString, VCPluginContext* out) {
    try {
        if (!out) return VC_INVALID;
        *out = nullptr;
        if (!host || host->abi_version != VC_ABI_VERSION || host->struct_size < sizeof(VCHostApi) || !host->invoke) return VC_INVALID;
        *out = new Context{*host};
        return VC_OK;
    } catch (...) { return VC_ERROR; }
}
static int32_t VC_CALL lifecycle(VCPluginContext ptr) { return ptr ? VC_OK : VC_INVALID; }
// 回复引用的借用数据与本地容器全部持续到 sink 返回。
static int32_t VC_CALL invoke(VCPluginContext ptr, const VCPluginRequest* request, VCReplySink sink, void* out) {
    try {
        if (!ptr || !request || !sink || request->header.abi_version != VC_ABI_VERSION || request->header.struct_size < sizeof(VCPluginRequest)) return VC_INVALID;
        auto& ctx = *static_cast<Context*>(ptr);
        ++ctx.count;
        auto response = initialized<VCPluginResponse>();
        auto reply = initialized<VCResponse>();
        reply.kind = VC_RESPONSE_PLUGIN;
        reply.plugin = &response;
        response.payload_json = request->payload_json;
        const auto operation = copied(request->operation);
        std::string counter;
        if (operation == "counter") {
            counter = std::to_string(ctx.count);
            response.payload_json = bytes(counter);
        }
        VCDocument document{};
        VCDocumentEdit edit{};
        std::vector<VCParagraph> paragraphs;
        std::vector<VCSegment> segments;
        VCTextStyle style{};
        const std::string suffix = " [C++]";
        if (operation == "typed-transform" || operation == "transform") {
            if (!request->document || !request->message || !request->message->document || !request->message->source_id.len) return VC_INVALID;
            document = *request->message->document;
            document.revision = request->document->revision + 1;
            paragraphs = copied_slice<VCParagraph>(document.paragraphs);
            if (paragraphs.empty()) paragraphs.push_back(initialized<VCParagraph>());
            segments = copied_slice<VCSegment>(paragraphs.back().segments);
            style = initialized<VCTextStyle>();
            style.bold = 1;
            auto segment = initialized<VCSegment>();
            segment.text = text(suffix);
            segment.style = &style;
            segments.push_back(segment);
            paragraphs.back().segments = {segments.data(), static_cast<uint64_t>(segments.size()), sizeof(VCSegment)};
            document.paragraphs = {paragraphs.data(), static_cast<uint64_t>(paragraphs.size()), sizeof(VCParagraph)};
            edit = initialized<VCDocumentEdit>();
            edit.expected_revision = request->document->revision;
            edit.replacement = &document;
            response.edit = &edit;
        }
        return sink(out, &reply);
    } catch (...) { return VC_ERROR; }
}
static int32_t VC_CALL save(VCPluginContext ptr, VCReplySink sink, void* out) {
    try {
        if (!ptr || !sink) return VC_INVALID;
        const std::string id = "example.cpp";
        const std::string count = std::to_string(static_cast<Context*>(ptr)->count);
        auto state = initialized<VCStateSnapshot>();
        state.plugin_id = text(id);
        state.format_version = 1;
        state.data = bytes(count);
        auto reply = initialized<VCResponse>();
        reply.kind = VC_RESPONSE_SNAPSHOT;
        reply.snapshot = &state;
        return sink(out, &reply);
    } catch (...) { return VC_ERROR; }
}
static int32_t VC_CALL restore(VCPluginContext ptr, const VCStateSnapshot* state) {
    try {
        if (!ptr || !state || state->header.abi_version != VC_ABI_VERSION || state->header.struct_size < sizeof(VCStateSnapshot) || state->format_version != 1 || copied(state->plugin_id) != "example.cpp") return VC_INVALID;
        if (state->data.len > VC_MAX_BYTES || state->data.stride != 1 || (state->data.len && !state->data.data)) return VC_INVALID;
        const std::string value(reinterpret_cast<const char*>(state->data.data), static_cast<size_t>(state->data.len));
        size_t end = 0;
        auto count = std::stoull(value, &end);
        if (end != value.size()) return VC_INVALID;
        static_cast<Context*>(ptr)->count = count;
        return VC_OK;
    } catch (...) { return VC_ERROR; }
}
static int32_t VC_CALL destroy(VCPluginContext ptr) {
    try { delete static_cast<Context*>(ptr); return VC_OK; } catch (...) { return VC_ERROR; }
}
static const VCPluginApi api{VC_ABI_VERSION, sizeof(VCPluginApi), create, lifecycle, invoke, lifecycle, lifecycle, save, restore, destroy};
extern "C" VC_EXPORT const VCPluginApi* VC_CALL vchat_internal_plugin_get_api(void) { return &api; }
