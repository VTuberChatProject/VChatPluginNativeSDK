/* VChat C ABI 2：借用视图、同步深拷贝回复；开发库由测试宿主使用。 */
#ifndef VC_PLUGIN_INTERFACE_H
#define VC_PLUGIN_INTERFACE_H
#include <stddef.h>
#include <stdint.h>
#ifdef _WIN32
#define VC_CALL __cdecl
#define VC_EXPORT __declspec(dllexport)
#else
#define VC_CALL
#define VC_EXPORT __attribute__((visibility("default")))
#endif
#ifdef __cplusplus
extern "C" {
#endif
#define VC_ABI_VERSION 2u
#define VC_MODEL_VERSION 1u
#define VC_MAX_BYTES (16u * 1024u * 1024u)
#define VC_MAX_DEPTH 64u
enum { VC_OK=0, VC_ERROR=1, VC_STALE=2, VC_INVALID=3, VC_UNSUPPORTED=4, VC_FORBIDDEN=5, VC_BUSY=6, VC_TIMEOUT=7 };
enum { VC_RESPONSE_PLUGIN=1, VC_RESPONSE_SNAPSHOT=2, VC_RESPONSE_HOST=3 };
enum { VC_HOST_EXTENSION=0, VC_HOST_MESSAGE=1, VC_HOST_TYPES=2, VC_HOST_UI=3, VC_HOST_FRONTEND=4, VC_HOST_WEB=5, VC_HOST_PATHS=6, VC_HOST_LOG=7, VC_HOST_SUBSCRIPTION=8, VC_HOST_WINDOW=9 };
enum { VC_PLUGIN_EXTENSION=0, VC_PLUGIN_TRANSFORM=1, VC_PLUGIN_EVENT=2, VC_PLUGIN_FRONTEND=3, VC_PLUGIN_WEB=4 };
typedef void *VCPluginContext;
typedef void *VCHostContext;
typedef struct { uint32_t abi_version, struct_size; } VCHeader;
typedef struct { const uint8_t *data; uint64_t len; } VCString;
typedef struct { uint32_t has_value, value; } VCOptionalU32;
typedef struct { uint32_t has_value; uint64_t value; } VCOptionalU64;
typedef struct { uint32_t has_value; int64_t value; } VCOptionalI64;
typedef struct VCDocument VCDocument;
typedef struct VCParagraph VCParagraph;
typedef struct VCSegment VCSegment;
typedef struct VCTextStyle VCTextStyle;
typedef struct VCImage VCImage;
typedef struct VCMessage VCMessage;
typedef struct VCMessageType VCMessageType;
typedef struct VCSelection VCSelection;
typedef struct VCPluginError VCPluginError;
typedef struct VCDocumentEdit VCDocumentEdit;
typedef struct VCPluginResponse VCPluginResponse;
typedef struct VCUiCommand VCUiCommand;
typedef struct VCUiField VCUiField;
typedef struct VCOutputAction VCOutputAction;
typedef struct VCOutputCard VCOutputCard;
typedef struct VCUiContribution VCUiContribution;
typedef struct VCStateSnapshot VCStateSnapshot;
typedef struct VCMessageInput VCMessageInput;
typedef struct VCMessageTypeDefinition VCMessageTypeDefinition;
typedef struct VCMessageTypes VCMessageTypes;
typedef struct VCRuntimePaths VCRuntimePaths;
typedef struct VCFrontendPayload VCFrontendPayload;
typedef struct VCHeaderPair VCHeaderPair;
typedef struct VCWebRoute VCWebRoute;
typedef struct VCWebPayload VCWebPayload;
typedef struct VCEventPayload VCEventPayload;
typedef struct VCLogPayload VCLogPayload;
typedef struct VCSubscription VCSubscription;
typedef struct VCWindowPayload VCWindowPayload;
typedef struct VCTransformPayload VCTransformPayload;
typedef struct VCHostRequest VCHostRequest;
typedef struct VCPluginRequest VCPluginRequest;
typedef struct VCResponse VCResponse;
typedef struct VCHostApi VCHostApi;
typedef struct VCPluginApi VCPluginApi;
typedef struct { const uint8_t *data; uint64_t len, stride; } VCBytes;
typedef struct { const VCParagraph *data; uint64_t len, stride; } VCParagraphSlice;
typedef struct { const VCSegment *data; uint64_t len, stride; } VCSegmentSlice;
typedef struct { const VCString *data; uint64_t len, stride; } VCStrings;
typedef struct { const VCOutputAction *data; uint64_t len, stride; } VCOutputActionSlice;
typedef struct { const VCUiCommand *data; uint64_t len, stride; } VCUiCommandSlice;
typedef struct { const VCUiField *data; uint64_t len, stride; } VCUiFieldSlice;
typedef struct { const VCOutputCard *data; uint64_t len, stride; } VCOutputCardSlice;
typedef struct { const VCMessageTypeDefinition *data; uint64_t len, stride; } VCMessageTypeDefinitionSlice;
typedef struct { const VCHeaderPair *data; uint64_t len, stride; } VCHeaderPairSlice;
typedef struct { const VCWebRoute *data; uint64_t len, stride; } VCWebRouteSlice;
struct VCDocument {
    VCHeader header;
    uint32_t version;
    uint64_t revision;
    VCParagraphSlice paragraphs;
};
struct VCParagraph {
    VCHeader header;
    VCSegmentSlice segments;
};
struct VCSegment {
    VCHeader header;
    VCString text;
    const VCTextStyle * style;
    const VCString * ruby;
    VCOptionalU32 speed;
    VCOptionalU32 pause;
    const VCImage * image;
    const VCString * class_name;
    VCBytes extra_json;
};
struct VCTextStyle {
    VCHeader header;
    uint32_t bold;
    uint32_t italic;
    uint32_t underline;
    uint32_t strikethrough;
    const VCString * color;
    const VCString * background_color;
    const VCString * size;
    VCBytes extra_json;
};
struct VCImage {
    VCHeader header;
    VCString source;
    float width;
    float height;
};
struct VCMessage {
    VCHeader header;
    VCString id;
    VCString source_id;
    const VCString * source_message_id;
    int64_t received_at_ms;
    uint64_t sequence;
    const VCDocument * document;
    VCOptionalI64 source_time;
    VCString username;
    VCString status;
    VCStrings origin_ids;
    const VCMessageType * message_type;
    const VCString * avatar_url;
};
struct VCMessageType {
    VCHeader header;
    VCString plugin_kind;
    VCString plugin_id;
    VCString type_id;
};
struct VCSelection {
    VCHeader header;
    uint64_t start;
    uint64_t end;
};
struct VCPluginError {
    VCHeader header;
    VCString code;
    VCString message;
};
struct VCDocumentEdit {
    VCHeader header;
    uint64_t expected_revision;
    const VCDocument * replacement;
};
struct VCPluginResponse {
    VCHeader header;
    VCBytes payload_json;
    const VCPluginError * error;
    const VCDocumentEdit * edit;
    const VCUiContribution * ui;
    uint32_t status;
    VCString content_type;
    uint32_t no_store;
    VCBytes body;
};
struct VCUiCommand {
    VCHeader header;
    VCString id;
    VCString label;
    VCString icon;
    VCString tooltip;
    uint32_t enabled;
    uint32_t selected;
};
struct VCUiField {
    VCHeader header;
    VCString id;
    VCString label;
    VCString kind;
    VCString value_json;
    VCStrings options;
    double minimum;
    double maximum;
    VCString schema_json;
};
struct VCOutputAction {
    VCHeader header;
    VCString id;
    VCString label;
    VCString kind;
    VCString target;
};
struct VCOutputCard {
    VCHeader header;
    VCString id;
    VCString title;
    VCString subtitle;
    VCOutputActionSlice actions;
};
struct VCUiContribution {
    VCHeader header;
    VCUiCommandSlice commands;
    VCUiFieldSlice fields;
    VCOutputCardSlice output_cards;
};
struct VCStateSnapshot {
    VCHeader header;
    VCString plugin_id;
    uint32_t format_version;
    VCBytes data;
};
struct VCMessageInput {
    VCHeader header;
    VCString source_id;
    const VCString * source_message_id;
    VCString username;
    VCOptionalI64 source_time;
    const VCString * message_type;
    const VCString * avatar_url;
    VCStrings origin_ids;
    const VCDocument * document;
    VCStrings present;
    VCBytes extra_json;
};
struct VCMessageTypeDefinition {
    VCHeader header;
    VCString id;
    VCString label;
};
struct VCMessageTypes {
    VCHeader header;
    uint32_t version;
    VCMessageTypeDefinitionSlice types;
};
struct VCRuntimePaths {
    VCHeader header;
    VCString package_root;
    VCString resources_root;
    VCString data_root;
    const VCString * assets_root;
};
struct VCFrontendPayload {
    VCHeader header;
    uint32_t version;
    VCString frontend_id;
    VCString message_id;
    VCString batch_id;
    VCString run_id;
    VCOptionalU64 generation;
    VCString label;
    VCString kind;
    VCString status;
    VCStrings capabilities;
    const VCMessage * message;
    VCStrings present;
    VCBytes extra_json;
};
struct VCHeaderPair {
    VCHeader header;
    VCString name;
    VCString value;
};
struct VCWebRoute {
    VCHeader header;
    VCString kind;
    VCString method;
    VCString path;
    VCString operation;
};
struct VCWebPayload {
    VCHeader header;
    uint32_t version;
    VCString method;
    VCString path;
    VCString query;
    VCString connection_id;
    uint32_t preview;
    VCHeaderPairSlice headers;
    VCBytes body;
    VCWebRouteSlice routes;
    VCBytes data_json;
    VCStrings present;
    VCBytes extra_json;
};
struct VCEventPayload {
    VCHeader header;
    uint32_t version;
    VCString event_id;
    VCString kind;
    const VCMessage * message;
    VCBytes data_json;
};
struct VCLogPayload {
    VCHeader header;
    VCString message;
    VCString level;
    VCBytes context_json;
};
struct VCSubscription {
    VCHeader header;
    VCStrings events;
};
struct VCWindowPayload {
    VCHeader header;
    uint32_t version;
    VCString window_id;
    VCString title;
    VCBytes content_json;
    VCBytes properties_json;
};
struct VCTransformPayload {
    VCHeader header;
    uint32_t version;
    const VCMessage * message;
    const VCDocument * document;
    uint32_t wrapped;
};
/* kind: 0=扩展/配置，1=消息，2=类型，3=UI，4=前端，5=Web，6=路径，7=日志，8=订阅，9=窗口。 */
struct VCHostRequest {
    VCHeader header; uint32_t kind; VCString service; VCBytes payload_json;
    const VCUiContribution *ui; VCString event_id;
    const VCMessageInput *message; const VCMessageTypes *message_types;
    const VCFrontendPayload *frontend; const VCWebPayload *web;
    const VCLogPayload *log; const VCSubscription *subscription; const VCWindowPayload *window;
};
/* kind: 0=扩展/配置/UI，1=转换，2=事件，3=前端，4=Web。 */
struct VCPluginRequest {
    VCHeader header; uint32_t kind, protocol_version; VCString operation; VCBytes payload_json;
    const VCDocument *document; const VCSelection *selection; VCString event_id; const VCMessage *message;
    const VCFrontendPayload *frontend; const VCWebPayload *web; const VCEventPayload *event; const VCTransformPayload *transform;
};
struct VCResponse {
    VCHeader header; int32_t status; uint32_t kind; VCString error_message;
    const VCPluginResponse *plugin; const VCStateSnapshot *snapshot; VCBytes bytes; const VCRuntimePaths *paths;
};
typedef int32_t (VC_CALL *VCReplySink)(void *, const VCResponse *);
typedef int32_t (VC_CALL *VCHostInvoke)(VCHostContext, const VCHostRequest *, VCReplySink, void *);
struct VCHostApi {
    uint32_t abi_version, struct_size; uint64_t generation; VCHostContext context; VCHostInvoke invoke;
};
typedef int32_t (VC_CALL *VCLifecycle)(VCPluginContext);
struct VCPluginApi {
    uint32_t abi_version, struct_size;
    int32_t (VC_CALL *create)(const VCHostApi *, VCString, VCPluginContext *);
    VCLifecycle start;
    int32_t (VC_CALL *invoke)(VCPluginContext, const VCPluginRequest *, VCReplySink, void *);
    VCLifecycle quiesce, resume;
    int32_t (VC_CALL *save_state)(VCPluginContext, VCReplySink, void *);
    int32_t (VC_CALL *restore_state)(VCPluginContext, const VCStateSnapshot *);
    VCLifecycle destroy;
};
/* 所有输入借用到函数返回，输出在 sink 内深拷贝；每次调用恰好回复一次。 */
VC_EXPORT const VCPluginApi *VC_CALL vchat_internal_plugin_get_api(void);
/* 开发动态库：backend 在 destroy 前有效；函数表与实例绑定，排空回调后销毁。 */
uint32_t VC_CALL vc_interface_get_version(void);
int32_t VC_CALL vc_interface_create_host(const VCHostApi *backend, VCHostApi **out);
int32_t VC_CALL vc_interface_destroy_host(VCHostApi *host);
#ifdef __cplusplus
}
static_assert(sizeof(VCHeader)==8, "VCHeader layout");
static_assert(offsetof(VCPluginApi,create)==8, "VCPluginApi prefix");
#else
_Static_assert(sizeof(VCHeader)==8, "VCHeader layout");
_Static_assert(offsetof(VCPluginApi,create)==8, "VCPluginApi prefix");
#endif
#endif
