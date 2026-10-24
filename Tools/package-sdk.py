"""构建独立开发 SDK，收集接口动态库、可离线定位的源码和跨语言契约。"""
import argparse
import json
import pathlib
import platform
import re
import shutil
import subprocess
import sys


def checked(command, root):
    """保留构建日志并在命令失败时中止发布。"""
    print("SDK:", " ".join(map(str, command)), flush=True)
    subprocess.run(command, cwd=root, check=True)


def source_tree(source, destination, suffixes):
    """只收集源文件，避免将构建缓存和开发环境混入 SDK。"""
    for item in source.rglob("*"):
        if item.is_file() and item.suffix in suffixes:
            target = destination / item.relative_to(source)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(item, target)


def main():
    """从仓库构建同源动态接口，并生成可独立构建的 SDK workspace。"""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-build", action="store_true", help="使用已有 release 构建产物")
    parser.add_argument("--output", type=pathlib.Path, help="指定一个空 SDK 输出目录")
    arguments = parser.parse_args()
    root = pathlib.Path(__file__).resolve().parents[2]
    system = {"Windows": "windows", "Linux": "linux", "Darwin": "macos"}[platform.system()]
    machine = platform.machine().lower()
    architecture = {"amd64": "x86_64", "arm64": "aarch64"}.get(machine, machine)
    output = arguments.output or root / f"target/dist/VCPluginSDK-{system}-{architecture}"
    output = output.resolve()
    if output.exists() and any(output.iterdir()):
        raise RuntimeError(f"请使用空 SDK 输出目录: {output}")
    if not arguments.skip_build:
        checked(["cargo", "build", "--locked", "--release", "-p", "vchat-plugin-api",
                 "--features", "sdk-dev"], root)
    library, standard = {
        "windows": ("vchat_plugin_api.dll", "VCPluginInterface.dll"),
        "linux": ("libvchat_plugin_api.so", "libVCPluginInterface.so"),
        "macos": ("libvchat_plugin_api.dylib", "libVCPluginInterface.dylib"),
    }[system]
    metadata = subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=root)
    release = pathlib.Path(json.loads(metadata)["target_directory"]) / "release"
    source_library = release / library
    if not source_library.is_file():
        raise RuntimeError(f"缺少开发库: {source_library}")
    output.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source_library, output / standard)
    # SDK 测试宿主显式装载库文件并解析导出，避免重命名后的 import/SONAME 绑定。
    for symbols in (release / "vchat_plugin_api.pdb", release / "libvchat_plugin_api.dylib.dSYM"):
        if symbols.is_dir():
            shutil.copytree(symbols, output / symbols.name)
        elif symbols.is_file():
            shutil.copy2(symbols, output / symbols.name)
    shutil.copy2(root / "Core/Interfaces/VCPluginInterface.h", output / "VCPluginInterface.h")
    (output / "Docs").mkdir()
    shutil.copy2(root / "Docs/PluginSystemDesign.md", output / "Docs/PluginSystemDesign.md")
    shutil.copy2(root / "LICENSE", output / "LICENSE")
    shutil.copy2(root / "THIRD_PARTY.md", output / "THIRD_PARTY.md")
    for filename in ("lib.rs", "Cargo.toml"):
        target = output / "Core" / filename
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(root / "Core" / filename, target)
    for directory in ("Models", "Services", "Interfaces", "Tests"):
        source_tree(root / "Core" / directory, output / "Core" / directory,
                    {".rs", ".toml", ".h", ".proto", ".md"})
    workspace = (root / "Cargo.toml").read_text(encoding="utf-8")
    dependencies = re.search(r"(?ms)^\[workspace.dependencies\]\n(.*?)(?=^\[)", workspace).group(1)
    (output / "Cargo.toml").write_text(
        '[workspace]\nresolver = "2"\nmembers = ["Core", "Core/Interfaces", "Plugins", '
        '"Plugins/EchoLiveConfig", "Plugins/NanaDialogue", "Plugins/VTypewriter", '
        '"Adapter/InternalPlugin"]\n\n'
        "[workspace.dependencies]\n" + dependencies, encoding="utf-8")
    source_tree(root / "Plugins", output / "Plugins",
                {".rs", ".h", ".hpp", ".cpp", ".toml", ".txt", ".json", ".js", ".cjs"})
    # 示例的动态测试与开发打包工具共享内部适配器，保持所有 path 依赖完整。
    source_tree(root / "Adapter/InternalPlugin", output / "Adapter/InternalPlugin",
                {".rs", ".toml", ".h", ".cpp", ".c", ".json"})
    shutil.copy2(root / "Cargo.lock", output / "Cargo.lock")
    (output / "README.md").write_text(
        "# VCPluginInterface SDK\n\n"
        "本目录包含 ABI 2 头文件、开发接口动态库、测试宿主和 Rust/C++ 示例。\n\n"
        "- 完整规范：[插件系统设计](Docs/PluginSystemDesign.md)。\n"
        "- 使用说明：[SDK](Core/Interfaces/SDK.md)。\n"
        "- Rust 示例：`cargo build -p vchat-example-plugin`。\n"
        "- C++ 示例：`cmake -S Plugins -B target/cpp-plugin -DVCHAT_SDK=.`。\n"
        "- 测试宿主：`python sdk-host.py --help`。\n\n"
        "发行插件通过宿主注入的函数表工作，开发动态库用于测试宿主。\n",
        encoding="utf-8")
    protocol = root / "Adapter/HttpPlugin/external_plugin.proto"
    if protocol.is_file():
        shutil.copy2(protocol, output / protocol.name)
        protocol_target = output / "Adapter/HttpPlugin" / protocol.name
        protocol_target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(protocol, protocol_target)
    for tool in ("sdk-host.py", "sdk-client.py", "sdk-client.cpp",
                 "package-sdk.py", "package-sdk.ps1", "package-sdk.sh"):
        source = root / "Core/Tools" / tool
        if source.is_file():
            tools_target = output / "Core/Tools" / tool
            tools_target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, tools_target)
            if tool.startswith("sdk-"):
                shutil.copy2(source, output / tool)
    (output / "sdk.json").write_text(json.dumps({
        "sdk_version": "0.2.0", "abi_version": 2, "external_protocol": 3,
        "platform": system, "architecture": architecture, "development_library": standard,
        "loading": "explicit-symbols", "plugin_runtime": "injected-host-api",
    }, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"SDK 打包完成: {output}")


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"SDK 打包失败: {error}", file=sys.stderr)
        raise SystemExit(1)
