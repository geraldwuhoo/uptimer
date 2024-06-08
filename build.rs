fn main() {
    let path = "./go";
    let lib = "shoutrrr";

    println!("cargo:rustc-link-search=native={}", path);
    println!("cargo:rustc-link-lib=static={}", lib);
}
