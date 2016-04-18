#[cfg(target_os = "windows")]
mod build{
	fn try_env(var: &str, path: &str) -> Option<String> {
		use std::env;
		match env::var(var) {
			Ok(var) => Some(var+path),
			Err(_) => None,
		}
	}

	pub fn build(){
		let path = try_env("CUDA_LIB_PATH","")
		.or(try_env("AMDAPPSDKROOT","/lib/x86_64"))
		.or(try_env("INTELOCLSDKROOT","/lib/x64"));
		match path {
			Some(path)=>{
				println!("cargo:rustc-link-search=native={}",path);
			},
			None=>println!("Error: cannot find env vars"),
		}
	}
}

#[cfg(not(target_os = "windows"))]
mod build{
	pub fn build(){

	}
}

fn main() {
	build::build();
}