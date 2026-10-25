"""构建并打包本仓库的插件开发 SDK，收集独立源码、头文件、工具和示例。"""
import argparse
import json
import logging
import pathlib
import platform
import shutil
import subprocess
import sys


def checked(command, root):
    """记录命令并在构建失败时终止打包。"""
    logging.info("执行: %s", " ".join(map(str, command)))
    subprocess.run(command, cwd=root, check=True)


def main():
    """从独立 workspace 构建开发库，并生成可重新构建的 SDK 发行目录。"""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-build", action="store_true", help="使用已有 release 开发库")
    parser.add_argument("--output", type=pathlib.Path, help="指定空 SDK 输出目录")
    arguments = parser.parse_args()
    root = pathlib.Path(__file__).resolve().parents[1]
    system = {"Windows": "windows", "Linux": "linux", "Darwin": "macos"}[platform.system()]
    machine = platform.machine().lower()
    architecture = {"amd64": "x86_64", "arm64": "aarch64"}.get(machine, machine)
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], cwd=root))
    target = pathlib.Path(metadata["target_directory"])
    output = (arguments.output or target / f"dist/VCPluginSDK-{system}-{architecture}").resolve()
    if output.exists() and any(output.iterdir()):
        raise RuntimeError(f"请指定空 SDK 输出目录: {output}")
    if not arguments.skip_build:
        checked(["cargo", "build", "--locked", "--release", "-p", "vchat-plugin-api",
                 "--features", "sdk-dev"], root)
    library, standard = {
        "windows": ("vchat_plugin_api.dll", "VCPluginInterface.dll"),
        "linux": ("libvchat_plugin_api.so", "libVCPluginInterface.so"),
        "macos": ("libvchat_plugin_api.dylib", "libVCPluginInterface.dylib"),
    }[system]
    source = target / "release" / library
    if not source.is_file():
        raise RuntimeError(f"缺少开发库: {source}")
    output.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, output / standard)
    for name in ("Rust", "Examples", "Tools", "Docs"):
        shutil.copytree(root / name, output / name,
                        ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
    for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "LICENSE", "README.md"):
        shutil.copy2(root / name, output / name)
    shutil.copy2(root / "Rust/VCPluginInterface.h", output / "VCPluginInterface.h")
    for symbols in (target / "release/vchat_plugin_api.pdb",
                    target / "release/libvchat_plugin_api.dylib.dSYM"):
        if symbols.is_dir():
            shutil.copytree(symbols, output / symbols.name)
        elif symbols.is_file():
            shutil.copy2(symbols, output / symbols.name)
    (output / "sdk.json").write_text(json.dumps({
        "sdk_version": "0.2.0", "abi_version": 2, "business_model_version": 1,
        "platform": system, "architecture": architecture,
        "development_library": standard, "loading": "explicit-symbols",
        "plugin_runtime": "injected-host-api",
    }, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    logging.info("SDK 打包完成: %s", output)


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format="%(levelname)s %(message)s")
    try:
        main()
    except (OSError, RuntimeError, subprocess.CalledProcessError):
        logging.exception("SDK 打包失败")
        raise SystemExit(1)
