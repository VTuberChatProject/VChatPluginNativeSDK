// 固定 Protobuf 编译器来源，构建时生成跨语言协议。
fn main() {
    println!("cargo:rerun-if-changed=internal_plugin.proto");
    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc_bin_vendored::protoc_bin_path().expect("protoc"));
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.field_attribute("vchat.plugin.v1.UiField.schema_json", "#[serde(default)]");
    config
        .compile_protos(&["internal_plugin.proto"], &["."])
        .expect("plugin protocol");
}
