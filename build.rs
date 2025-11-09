fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.contains("windows") {
        println!("cargo:rerun-if-changed=build/windows/icon.rc");
        println!("cargo:rerun-if-changed=build/windows/icon.ico");
        embed_resource::compile("build/windows/icon.rc", embed_resource::NONE).manifest_required().unwrap();
    }
}
