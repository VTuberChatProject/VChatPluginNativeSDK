"""显式加载开发接口库与真实插件，验证函数表注入及完整生命周期。"""
import argparse
import ctypes as c
import logging
import pathlib
import sys


class Header(c.Structure):
    """所有版本化视图的固定前缀。"""
    _fields_ = [("abi_version", c.c_uint32), ("struct_size", c.c_uint32)]


class StringView(c.Structure):
    """按字节长度借用的 UTF-8 字符串。"""
    _fields_ = [("data", c.c_void_p), ("len", c.c_uint64)]


class ResponsePrefix(c.Structure):
    """测试宿主只检查回复的稳定状态前缀。"""
    _fields_ = [("header", Header), ("status", c.c_int32), ("kind", c.c_uint32)]


class BytesView(c.Structure):
    """回复中固定布局的字节视图。"""
    _fields_ = [("data", c.c_void_p), ("len", c.c_uint64), ("stride", c.c_uint64)]


class Response(c.Structure):
    """模拟后端同样使用完整回复布局和一次同步 sink。"""
    _fields_ = [("header", Header), ("status", c.c_int32), ("kind", c.c_uint32),
                ("error_message", StringView), ("plugin", c.c_void_p),
                ("snapshot", c.c_void_p), ("bytes", BytesView), ("paths", c.c_void_p)]


Reply = c.CFUNCTYPE(c.c_int32, c.c_void_p, c.c_void_p)
HostInvoke = c.CFUNCTYPE(c.c_int32, c.c_void_p, c.c_void_p, Reply, c.c_void_p)


class HostApi(c.Structure):
    """与 VCPluginInterface.h 一致的宿主 API 表。"""
    _fields_ = [("abi_version", c.c_uint32), ("struct_size", c.c_uint32),
                ("generation", c.c_uint64), ("context", c.c_void_p), ("invoke", HostInvoke)]


Create = c.CFUNCTYPE(c.c_int32, c.POINTER(HostApi), StringView, c.POINTER(c.c_void_p))
Lifecycle = c.CFUNCTYPE(c.c_int32, c.c_void_p)
Invoke = c.CFUNCTYPE(c.c_int32, c.c_void_p, c.c_void_p, Reply, c.c_void_p)
Save = c.CFUNCTYPE(c.c_int32, c.c_void_p, Reply, c.c_void_p)
Restore = c.CFUNCTYPE(c.c_int32, c.c_void_p, c.c_void_p)


class PluginApi(c.Structure):
    """插件拥有的只读生命周期函数表。"""
    _fields_ = [("abi_version", c.c_uint32), ("struct_size", c.c_uint32),
                ("create", Create), ("start", Lifecycle), ("invoke", Invoke),
                ("quiesce", Lifecycle), ("resume", Lifecycle), ("save_state", Save),
                ("restore_state", Restore), ("destroy", Lifecycle)]


def ensure(status, operation):
    """记录每一步 ABI 操作并传播失败。"""
    if status:
        raise RuntimeError(f"{operation} 失败，ABI 状态码 {status}")
    logging.info("%s 成功", operation)


def run(library_path, plugin_path):
    """同时持有动态库与回调直到实例排空，验证开发库与插件真实互操作。"""
    interface = c.CDLL(str(library_path.resolve(strict=True)))
    interface.vc_interface_get_version.restype = c.c_uint32
    if interface.vc_interface_get_version() != 2:
        raise RuntimeError("开发库 ABI 版本需要为 2")
    interface.vc_interface_create_host.argtypes = [c.POINTER(HostApi), c.POINTER(c.POINTER(HostApi))]
    interface.vc_interface_create_host.restype = c.c_int32
    interface.vc_interface_destroy_host.argtypes = [c.POINTER(HostApi)]
    interface.vc_interface_destroy_host.restype = c.c_int32

    @HostInvoke
    def mock(context, request, reply, output):
        logging.warning("此生命周期测试后端返回 unsupported；业务测试请注入实际服务后端")
        message = c.create_string_buffer(b"SDK mock backend: unsupported service")
        response = Response(Header(2, c.sizeof(Response)), 4, 3,
                            StringView(c.cast(message, c.c_void_p), len(message.value)),
                            None, None, BytesView(None, 0, 1), None)
        status = reply(output, c.byref(response))
        return status or 4

    backend = HostApi(2, c.sizeof(HostApi), 1, c.c_void_p(1), mock)
    host = c.POINTER(HostApi)()
    ensure(interface.vc_interface_create_host(c.byref(backend), c.byref(host)), "创建开发宿主")
    context = c.c_void_p()
    api = None
    started = False
    try:
        plugin = c.CDLL(str(plugin_path.resolve(strict=True)))
        plugin.vchat_internal_plugin_get_api.restype = c.POINTER(PluginApi)
        pointer = plugin.vchat_internal_plugin_get_api()
        if not pointer:
            raise RuntimeError("插件返回空 API 表")
        prefix = c.cast(pointer, c.POINTER(Header)).contents
        if prefix.abi_version != 2 or prefix.struct_size < c.sizeof(PluginApi):
            raise RuntimeError("插件 ABI 版本或函数表大小无效")
        api = pointer.contents
        config = c.create_string_buffer(b"{}")
        ensure(api.create(host, StringView(c.cast(config, c.c_void_p), 2), c.byref(context)), "create")
        if not context:
            raise RuntimeError("create 成功时需要返回实例")
        ensure(api.start(context), "start")
        started = True
        ensure(api.quiesce(context), "quiesce")
        started = False
        responses = []

        @Reply
        def collect(user, response):
            if not response:
                return 3
            prefix = c.cast(response, c.POINTER(ResponsePrefix)).contents
            if prefix.header.abi_version != 2 or prefix.header.struct_size < c.sizeof(ResponsePrefix):
                return 3
            responses.append((prefix.status, prefix.kind))
            return 0

        ensure(api.save_state(context, collect, None), "save_state")
        if responses != [(0, 2)]:
            raise RuntimeError(f"快照需要恰好一次成功回复，实际 {responses}")
        ensure(api.resume(context), "resume")
        started = True
        ensure(api.quiesce(context), "再次 quiesce")
        started = False
        ensure(api.destroy(context), "destroy")
        context = c.c_void_p()
    finally:
        if context and api is not None:
            if started:
                ensure(api.quiesce(context), "失败清理 quiesce")
            ensure(api.destroy(context), "失败清理 destroy")
        ensure(interface.vc_interface_destroy_host(host), "销毁开发宿主")
    logging.info("开发动态库与真实插件生命周期验证通过")


def main():
    """选择本平台开发库；插件使用独立 SDK 的 C++ 示例产物。"""
    parser = argparse.ArgumentParser(description=__doc__)
    default_library = {"win32": "VCPluginInterface.dll", "darwin": "libVCPluginInterface.dylib"}.get(
        sys.platform, "libVCPluginInterface.so")
    parser.add_argument("--library", type=pathlib.Path,
                        default=pathlib.Path(__file__).resolve().parent / default_library)
    parser.add_argument("--plugin", required=True, type=pathlib.Path,
                        help="无需宿主业务服务即可启动的插件，例如 SDK 的 C++ 示例")
    arguments = parser.parse_args()
    logging.basicConfig(level=logging.INFO, format="%(levelname)s %(message)s")
    try:
        run(arguments.library, arguments.plugin)
    except Exception:
        logging.exception("开发接口验证失败")
        raise SystemExit(1)


if __name__ == "__main__":
    main()
