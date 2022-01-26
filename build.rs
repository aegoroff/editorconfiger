extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new()
        .always_use_colors()
        .process_current_dir()
        .expect("Glob parser compilation failed");
}
