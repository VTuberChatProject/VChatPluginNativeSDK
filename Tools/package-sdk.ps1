# 独立构建开发接口动态库并整理 SDK，主程序发行包使用内嵌实现。
param([switch]$SkipBuild, [string]$Output)
$ErrorActionPreference = 'Stop'
$sdkArguments = @((Join-Path $PSScriptRoot 'package-sdk.py'))
if ($SkipBuild) { $sdkArguments += '--skip-build' }
if ($Output) { $sdkArguments += @('--output', $Output) }
& python @sdkArguments
if ($LASTEXITCODE -ne 0) { throw "SDK 打包失败: $LASTEXITCODE" }
