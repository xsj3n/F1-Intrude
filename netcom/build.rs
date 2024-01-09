fn main()
{
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=lib.rs");
  cxx_build::bridge("src/lib.rs")  // returns a cc::Build/ 
  .compile("netcom");
}

