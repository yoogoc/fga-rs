extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new()
        .set_in_dir("src")
        .set_out_dir("src")
        .process_file("src/grammar.lalrpop")
        .unwrap();
    println!("cargo:rerun-if-changed=src/grammar.lalrpop");
}
