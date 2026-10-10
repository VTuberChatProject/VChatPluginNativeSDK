// C++ SDK 示例：所有异常在 C ABI 边界捕获，响应通过宿主写入回调。
#include "internal_plugin.h"
#include "internal_plugin.pb.h"
#include <memory>
#include <string>
#include <cstddef>
static_assert(offsetof(VChatHostApi, generation) == 8);
static_assert(offsetof(VChatPluginApi, create) == 8);
static_assert(sizeof(VChatPluginApi) == 8 + sizeof(void*) * 8);
namespace proto = vchat::plugin::v1;
struct Context { VChatHostApi host; uint64_t count = 0; };
static int32_t create(const VChatHostApi* host, const uint8_t*, size_t, void** out) {
    try {
        if (!host || !out || host->abi_version != 1 || host->struct_size < sizeof(VChatHostApi)) return 1;
        *out = new Context{*host}; return 0;
    } catch (...) { return 1; }
}
static int32_t lifecycle(void*) { return 0; }
static int32_t invoke(void* ptr, const uint8_t* data, size_t len, VChatWrite write, void* out) {
    try {
        proto::PluginRequest request;
        if (!request.ParseFromArray(data, static_cast<int>(len)) || request.protocol_version() != 1) return 1;
        auto& ctx = *static_cast<Context*>(ptr); ++ctx.count;
        proto::PluginResponse response;
        response.set_payload_json(request.operation() == "counter" ? std::to_string(ctx.count) : request.payload_json());
        if (request.operation() == "typed-transform") {
            if (!request.has_document() || !request.has_message() || !request.message().has_document() || request.message().source_id().empty()) return 1;
            auto* edit = response.mutable_edit();
            edit->set_expected_revision(request.document().revision());
            auto* document = edit->mutable_replacement();
            *document = request.message().document();
            document->set_revision(request.document().revision() + 1);
            if (document->paragraphs_size() == 0) document->add_paragraphs();
            auto* segment = document->mutable_paragraphs(document->paragraphs_size()-1)->add_segments();
            segment->set_text(" [C++]");
            segment->mutable_style()->set_bold(true);
        }
        auto bytes = response.SerializeAsString();
        return write(out, reinterpret_cast<const uint8_t*>(bytes.data()), bytes.size());
    } catch (...) { return 1; }
}
static int32_t save(void* ptr, VChatWrite write, void* out) {
    try {
        proto::StateSnapshot snapshot;
        snapshot.set_plugin_id("example.cpp"); snapshot.set_format_version(1);
        snapshot.set_data(std::to_string(static_cast<Context*>(ptr)->count));
        auto bytes = snapshot.SerializeAsString();
        return write(out, reinterpret_cast<const uint8_t*>(bytes.data()), bytes.size());
    } catch (...) { return 1; }
}
static int32_t restore(void* ptr, const uint8_t* data, size_t len) {
    try {
        proto::StateSnapshot snapshot;
        if (!snapshot.ParseFromArray(data, static_cast<int>(len)) || snapshot.format_version() != 1 || snapshot.plugin_id() != "example.cpp") return 1;
        static_cast<Context*>(ptr)->count = std::stoull(snapshot.data()); return 0;
    } catch (...) { return 1; }
}
static int32_t destroy(void* ptr) { try { delete static_cast<Context*>(ptr); return 0; } catch (...) { return 1; } }
static const VChatPluginApi api{1, sizeof(VChatPluginApi), create, lifecycle, invoke, lifecycle, lifecycle, save, restore, destroy};
extern "C" VCHAT_EXPORT const VChatPluginApi* vchat_internal_plugin_get_api(void) { return &api; }
