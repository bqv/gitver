extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use std::{env, io::{self, Read}};
use std::fs::File;

fn git_id() -> Result<String, io::Error> {
    let mut cwd = env::current_dir()?;
    loop {
        let gitdir = cwd.join(".git");
        if gitdir.is_dir() {
            let headfile = gitdir.join("HEAD");
            let head = {
                let mut file = File::open(headfile.as_path())?;
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
                Ok::<String, io::Error>(buffer)
            }?;
            if !head.starts_with("ref: ") {
                // Detached head
                return Ok(head);
            }
            let reffile = gitdir.join((&head[5..]).trim_end_matches(&['\r','\n'][..]));
            let oid = {
                let mut file = File::open(reffile.as_path())?;
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
                Ok::<String, io::Error>(buffer)
            }?;
            return Ok(oid.trim_end_matches(&['\r','\n'][..]).to_string());
        } else {
            cwd = cwd.join("..").canonicalize()?;
        }
    }
}

#[proc_macro]
pub fn gitver(_item: TokenStream) -> TokenStream {
    let def = "?.?.?".to_string();
    let vid = env::var("CARGO_PKG_VERSION").unwrap_or(def);
    let gid = match git_id() {
        Ok(id) => format!("{}-{}", vid, &id[0..7]),
        Err(err) => {
            println!("Git Error: {}", err);
            vid.to_string()
        }
    };
    quote!(
        fn gitver() -> &'static str {
            #gid
        }
    ).into()
}
