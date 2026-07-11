fn main() {
  let bytes = uppercut_core::compile_gain_wasm().unwrap();
  std::fs::write("examples/plugins/gain/gain.wasm", bytes).unwrap();
  println!("wrote {} bytes", std::fs::metadata("examples/plugins/gain/gain.wasm").unwrap().len());
}
