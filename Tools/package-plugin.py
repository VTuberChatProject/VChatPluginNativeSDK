"""将独立插件构建产物与资源整理为清单版本 2 的 .vcplug 包。"""
import argparse
import json
import logging
import pathlib
import platform
import shutil
import tempfile
import zipfile


def main():
    """按实际平台填写清单，保留插件身份和版本，归档到指定输出文件。"""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, type=pathlib.Path)
    parser.add_argument("--library", required=True, type=pathlib.Path)
    parser.add_argument("--resources", type=pathlib.Path)
    parser.add_argument("--output", required=True, type=pathlib.Path)
    args = parser.parse_args()
    library = args.library.resolve(strict=True)
    manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
    system = {"Windows": "windows", "Darwin": "macos", "Linux": "linux"}[platform.system()]
    machine = platform.machine().lower()
    manifest.update(package_version=2, platform=system,
                    architecture={"amd64": "x86_64", "arm64": "aarch64"}.get(machine, machine),
                    entry=f"bin/{library.name}")
    if manifest.get("abi_version") != 2:
        raise ValueError("插件清单需要声明 ABI 2")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="vchat-plugin-") as temporary:
        stage = pathlib.Path(temporary)
        (stage / "bin").mkdir()
        shutil.copy2(library, stage / "bin" / library.name)
        if args.resources:
            resources = args.resources.resolve(strict=True)
            for item in resources.rglob("*"):
                if item.is_symlink():
                    raise ValueError(f"资源包含链接: {item}")
            shutil.copytree(resources, stage / "resources", ignore=shutil.ignore_patterns(".git"))
        (stage / "manifest.json").write_text(
            json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        with zipfile.ZipFile(args.output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for item in sorted(stage.rglob("*")):
                if item.is_file():
                    archive.write(item, item.relative_to(stage).as_posix())
    logging.info("插件打包完成: %s", args.output.resolve())


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format="%(levelname)s %(message)s")
    try:
        main()
    except Exception:
        logging.exception("插件打包失败")
        raise SystemExit(1)
